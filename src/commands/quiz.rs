//! `quiz` commands — run/list/show/results.

use serde::Deserialize;
use serde_json::{json, Value};

use crate::core::error::CarpenterError;
use crate::core::exec;
use crate::core::{db, notebook, store, store::Paths};
use crate::models::quiz::{CaseResult, QuizListItem, QuizRunItem};
use crate::models::Data;

#[derive(Deserialize, Default)]
struct LastCheck {
    #[serde(default)]
    passed: i64,
    #[serde(default)]
    total: i64,
    #[serde(default)]
    cases: Vec<LastCheckCase>,
}

#[derive(Deserialize, Default)]
struct LastCheckCase {
    #[serde(default)]
    case_id: String,
    #[serde(default)]
    passed: i64,
    #[serde(default)]
    error: Option<String>,
}

/// Parse a stored `last_check` JSON blob (defensive: `{}`/bad → zeros + empty).
fn parse_last_check(blob: &str) -> (i64, i64, Vec<CaseResult>) {
    let lc: LastCheck = serde_json::from_str(blob).unwrap_or_default();
    let cases = lc
        .cases
        .into_iter()
        .map(|c| CaseResult {
            case_id: c.case_id,
            passed: c.passed != 0,
            error: c.error,
        })
        .collect();
    (lc.passed, lc.total, cases)
}

/// Build a `signature`/`prompt` map over a lesson's practice + quizzes.
fn scaffold_map(
    conn: &rusqlite::Connection,
    lesson_id: &str,
) -> Result<std::collections::HashMap<String, (String, String)>, CarpenterError> {
    let mut map = std::collections::HashMap::new();
    for q in db::list_quizzes(conn, lesson_id)? {
        map.insert(q.id, (q.signature, q.prompt));
    }
    for s in db::list_sections(conn, lesson_id)? {
        for p in db::list_practice(conn, &s.id)? {
            map.insert(p.id, (p.signature, p.prompt));
        }
    }
    Ok(map)
}

/// `quiz run` — execute the lesson notebook in the course venv; helper cells score
/// and write `pass_or_fail`/`last_check`. Scaffolding errors escalate; learner
/// errors are scored as fails.
pub fn run(
    paths: &Paths,
    course_slug: &str,
    lesson_id: &str,
    timeout: u64,
) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, lesson_id)?;
    let course_dir = paths.course(course_slug);
    if !course_dir.join(".venv").exists() {
        return Err(CarpenterError::StoreError(format!(
            "no course venv for {course_slug} — run `carpenter venv create` first"
        )));
    }
    let dir = crate::commands::lesson::lesson_dir(paths, course_slug, &lesson.slug, lesson.ord);
    let timeout_arg = format!("--ExecutePreprocessor.timeout={timeout}");
    // Run nbconvert from the lesson dir so the kernel cwd resolves `import helper`
    // (helper.py lives next to the notebook). `uv run` walks up to find the venv.
    let args = [
        "run",
        "jupyter",
        "nbconvert",
        "--execute",
        "--to",
        "notebook",
        "--inplace",
        "--ExecutePreprocessor.allow_errors=True",
        timeout_arg.as_str(),
        "lesson.ipynb",
    ];
    exec::run_uv_or_store(&args, &dir)?;

    // classify errored cells via scaffold_hash
    let nb_path = dir.join("lesson.ipynb");
    let nb_text = std::fs::read_to_string(&nb_path).map_err(store::io_to_store)?;
    let nb: Value = serde_json::from_str(&nb_text)
        .map_err(|e| CarpenterError::StoreError(format!("notebook unreadable: {e}")))?;
    let scaffolds = scaffold_map(&conn, lesson_id)?;
    let mut scaff_errors: Vec<Value> = Vec::new();
    for err in notebook::scan_errors(&nb) {
        let Some(cell) = nb["cells"].get(err.index) else {
            continue;
        };
        if let Some((_owner, id, stored_hash)) = notebook::stub_info(cell) {
            if let Some((sig, prompt)) = scaffolds.get(&id) {
                let canonical = notebook::scaffold_hash(sig, prompt);
                if canonical == stored_hash {
                    // scaffold unchanged but it errored ⇒ scaffolding bug
                    scaff_errors.push(json!({
                        "index": err.index,
                        "ename": err.ename,
                        "evalue": err.evalue,
                    }));
                }
            }
        }
    }
    if !scaff_errors.is_empty() {
        return Err(CarpenterError::ExecuteError {
            message: "scaffolding cell(s) errored — rewrite the section".into(),
            details: json!({ "errors": scaff_errors }),
        });
    }

    let mut quizzes = Vec::new();
    for q in db::list_quizzes(&conn, lesson_id)? {
        let (passed, total, cases) = parse_last_check(&q.last_check);
        quizzes.push(QuizRunItem {
            quiz_id: q.id,
            skipped: q.skip,
            pass_or_fail: q.pass_or_fail,
            passed,
            total,
            cases,
        });
    }
    Ok(Data::QuizRun {
        lesson_id: lesson_id.into(),
        quizzes,
        saved: true,
    })
}

/// `quiz list` (optionally filtered to a lesson).
pub fn list(
    paths: &Paths,
    course_slug: &str,
    lesson_id: Option<&str>,
) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let mut out = Vec::new();
    let lessons: Vec<String> = match lesson_id {
        Some(l) => vec![l.into()],
        None => db::list_lessons(&conn)?.into_iter().map(|l| l.id).collect(),
    };
    for lid in lessons {
        for q in db::list_quizzes(&conn, &lid)? {
            out.push(QuizListItem {
                case_count: db::owner_case_count(&conn, "quiz", &q.id)?,
                id: q.id,
                lesson_id: lid.clone(),
                name: q.name,
                skip: q.skip,
                pass_or_fail: q.pass_or_fail,
            });
        }
    }
    Ok(Data::QuizList { quizzes: out })
}

/// `quiz show <quiz_id>`.
pub fn show(paths: &Paths, course_slug: &str, quiz_id: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let q = db::get_quiz(&conn, quiz_id)?;
    Ok(Data::QuizShow {
        id: q.id,
        lesson_id: db::quiz_lesson_id(&conn, quiz_id)?,
        name: q.name,
        signature: q.signature,
        prompt: q.prompt,
        cases: db::owner_case_count(&conn, "quiz", quiz_id)?,
        skip: q.skip,
        pass_or_fail: q.pass_or_fail,
    })
}

/// `quiz results <quiz_id>` (live snapshot from `last_check`).
pub fn results(paths: &Paths, course_slug: &str, quiz_id: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let q = db::get_quiz(&conn, quiz_id)?;
    let (passed, total, cases) = parse_last_check(&q.last_check);
    Ok(Data::QuizResults {
        quiz_id: q.id,
        skipped: q.skip,
        pass_or_fail: q.pass_or_fail,
        passed,
        total,
        cases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    const LESSON: &str = "title: Arrays\nslug: arrays\nsections:\n  - title: s\n    snippets:\n      - kind: markdown\n        content: hi\n    practice:\n      - name: f\n        signature: \"def f(x):\"\n        cases:\n          - args: [1]\n            expected: 1\nquizzes:\n  - name: max_value\n    signature: \"def max_value(arr):\"\n    cases:\n      - args:\n          - [3, 1, 2]\n        expected: 3\n";

    fn setup_lesson() -> (Paths, String) {
        let (paths, slug) = testutil::setup();
        crate::commands::lesson::create(&paths, &slug, LESSON).expect("create lesson");
        (paths, slug)
    }

    #[test]
    fn list_ok() {
        let (paths, slug) = setup_lesson();
        let Data::QuizList { quizzes } = list(&paths, &slug, Some("arrays")).expect("list") else {
            panic!();
        };
        assert_eq!(quizzes.len(), 1);
        assert_eq!(quizzes[0].case_count, 1);
    }

    #[test]
    fn show_ok() {
        let (paths, slug) = setup_lesson();
        let Data::QuizShow { cases, .. } = show(&paths, &slug, "q1").expect("show") else {
            panic!();
        };
        assert_eq!(cases, 1);
    }

    #[test]
    fn results_ok_empty_before_run() {
        let (paths, slug) = setup_lesson();
        let Data::QuizResults { passed, total, .. } =
            results(&paths, &slug, "q1").expect("results")
        else {
            panic!();
        };
        // no run yet ⇒ last_check is '{}' ⇒ 0/0
        assert_eq!(passed, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn parse_last_check_handles_empty_and_real() {
        let (p, t, c) = parse_last_check("{}");
        assert_eq!((p, t), (0, 0));
        assert!(c.is_empty());
        let (p, t, c) = parse_last_check(
            r#"{"passed":1,"total":2,"cases":[{"case_id":"c1","passed":1},{"case_id":"c2","passed":0,"error":"ValueError: x"}]}"#,
        );
        assert_eq!((p, t), (1, 2));
        assert_eq!(c.len(), 2);
        assert!(c[0].passed);
        assert!(!c[1].passed);
        assert_eq!(c[1].error.as_deref(), Some("ValueError: x"));
    }

    #[test]
    #[ignore = "needs a course venv + nbconvert (run manually)"]
    fn run_executes_notebook() {
        let (paths, slug) = setup_lesson();
        crate::commands::venv::create(&paths, &slug, None).expect("venv");
        let _ = run(&paths, &slug, "arrays", 30).expect("run");
    }
}
