//! `lesson` commands — create/get (P4) + list/show/update/delete/sync (P5).

use std::fs;

use serde_json::{json, Map, Value};

use crate::core::compare::CompareMode;
use crate::core::error::CarpenterError;
use crate::core::exec;
use crate::core::store;
use crate::core::{db, notebook, status, store::Paths, time};
use crate::models::execute::{ExecError, ExecuteCells};
use crate::models::lesson::{
    CaseTree, CheckableTree, LessonCounts, LessonListItem, LessonProgress, LessonRow, LessonSpec,
    SectionTree,
};
use crate::models::Data;

fn validate_lesson_spec(spec: &LessonSpec) -> Result<(), CarpenterError> {
    if spec.title.trim().is_empty() {
        return Err(CarpenterError::ValidationError(
            "title must be non-empty".into(),
        ));
    }
    for (i, sec) in spec.sections.iter().enumerate() {
        if sec.snippets.is_empty() {
            return Err(CarpenterError::ValidationError(format!(
                "section {i} must have at least one snippet"
            )));
        }
        if sec.snippets[0].kind != "markdown" {
            return Err(CarpenterError::ValidationError(format!(
                "section {i}: snippets[0] must be markdown"
            )));
        }
    }
    Ok(())
}

fn unique_lesson_slug(conn: &rusqlite::Connection, base: &str) -> Result<String, CarpenterError> {
    if !db::lesson_slug_taken(conn, base)? {
        return Ok(base.to_string());
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !db::lesson_slug_taken(conn, &candidate)? {
            return Ok(candidate);
        }
        n += 1;
    }
}

fn serialize_snippets(
    snippets: &[crate::models::lesson::SnippetSpec],
    counter: &mut usize,
) -> String {
    let mut out = Vec::new();
    for s in snippets {
        *counter += 1;
        out.push(json!({
            "id": format!("sn{}", *counter),
            "kind": s.kind,
            "content": s.content,
        }));
    }
    serde_json::to_string(&out).unwrap_or_else(|_| String::from("[]"))
}

fn insert_cases(
    conn: &rusqlite::Connection,
    owner_type: &str,
    owner_id: &str,
    cases: &[crate::models::lesson::CaseSpec],
    total: &mut i64,
) -> Result<(), CarpenterError> {
    for (c_ord, case) in cases.iter().enumerate() {
        let cid = db::next_id(conn, "test_cases", "c")?;
        let args = serde_json::to_string(&case.args).unwrap_or_else(|_| String::from("[]"));
        let kwargs = serde_json::to_string(&case.kwargs).unwrap_or_else(|_| String::from("{}"));
        let expected =
            serde_json::to_string(&case.expected).unwrap_or_else(|_| String::from("null"));
        let compare = match case.compare {
            CompareMode::Exact => "exact",
            CompareMode::Sorted => "sorted",
            CompareMode::Set => "set",
        };
        db::insert_test_case(
            conn,
            &cid,
            owner_type,
            owner_id,
            &args,
            &kwargs,
            &expected,
            compare,
            c_ord as i64,
        )?;
        *total += 1;
    }
    Ok(())
}

/// Insert a lesson's sections/practice/quizzes/cases from a spec; return counts.
fn insert_lesson_content(
    conn: &rusqlite::Connection,
    lesson_id: &str,
    spec: &LessonSpec,
) -> Result<LessonCounts, CarpenterError> {
    let mut counts = LessonCounts {
        sections: 0,
        practice: 0,
        quizzes: 0,
        cases: 0,
    };
    let mut sn = 0usize;
    for (s_ord, section) in spec.sections.iter().enumerate() {
        let sid = db::next_id(conn, "sections", "s")?;
        let snippets_json = serialize_snippets(&section.snippets, &mut sn);
        db::insert_section(
            conn,
            &sid,
            lesson_id,
            &section.title,
            &snippets_json,
            s_ord as i64,
        )?;
        counts.sections += 1;
        for (p_ord, prac) in section.practice.iter().enumerate() {
            let pid = db::next_id(conn, "practice", "p")?;
            db::insert_practice(
                conn,
                &pid,
                &sid,
                &prac.name,
                &prac.signature,
                &prac.prompt,
                p_ord as i64,
            )?;
            counts.practice += 1;
            insert_cases(conn, "practice", &pid, &prac.cases, &mut counts.cases)?;
        }
    }
    for (q_ord, quiz) in spec.quizzes.iter().enumerate() {
        let qid = db::next_id(conn, "quizzes", "q")?;
        db::insert_quiz(
            conn,
            &qid,
            lesson_id,
            &quiz.name,
            &quiz.signature,
            &quiz.prompt,
            q_ord as i64,
        )?;
        counts.quizzes += 1;
        insert_cases(conn, "quiz", &qid, &quiz.cases, &mut counts.cases)?;
    }
    Ok(counts)
}

pub(crate) fn lesson_dir(paths: &Paths, course: &str, slug: &str, ord: i64) -> std::path::PathBuf {
    paths
        .course(course)
        .join("lessons")
        .join(format!("{ord:02}-{slug}"))
}

/// Create a lesson from a spec: DB inserts + render notebook + helper.
pub fn create(paths: &Paths, course_slug: &str, spec_json: &str) -> Result<Data, CarpenterError> {
    let spec: LessonSpec = store::parse_spec(spec_json)?;
    validate_lesson_spec(&spec)?;
    let conn = db::open_course(paths, course_slug)?;
    let now = time::now_iso();
    let base = match &spec.slug {
        Some(s) => s.clone(),
        None => store::slugify(&spec.title)?,
    };
    let slug = unique_lesson_slug(&conn, &base)?;
    let ord = spec
        .order
        .unwrap_or_else(|| db::next_lesson_ord(&conn).unwrap_or(1));
    let lesson_id = slug.clone();

    db::insert_lesson(&conn, &lesson_id, &slug, &spec.title, ord, &now, &now)?;
    let counts = insert_lesson_content(&conn, &lesson_id, &spec)?;

    let dir = lesson_dir(paths, course_slug, &slug, ord);
    fs::create_dir_all(&dir).map_err(store::io_to_store)?;
    let nb = notebook::render_to_string(&conn, &lesson_id)?;
    store::atomic_write(&dir.join("lesson.ipynb"), nb.as_bytes())?;
    store::atomic_write(
        &dir.join("helper.py"),
        crate::core::helper::HELPER_PY.as_bytes(),
    )?;

    Ok(Data::LessonCreate {
        id: lesson_id,
        slug,
        path: dir.display().to_string(),
        counts,
    })
}

/// Show the full lesson tree.
pub fn get(paths: &Paths, course_slug: &str, id: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, id)?;
    let status_str = status::lesson_status(&conn, id)?
        .map(|s| s.as_str().to_string())
        .unwrap_or(lesson.status);
    let mut sections = Vec::new();
    for s in db::list_sections(&conn, id)? {
        let mut practice = Vec::new();
        for p in db::list_practice(&conn, &s.id)? {
            practice.push(checkable_tree(&conn, "practice", &p)?);
        }
        sections.push(SectionTree {
            id: s.id,
            title: s.title,
            snippets: notebook::parse_snippets(&s.snippets),
            ord: s.ord,
            practice,
        });
    }
    let mut quizzes = Vec::new();
    for q in db::list_quizzes(&conn, id)? {
        quizzes.push(checkable_tree(&conn, "quiz", &q)?);
    }
    Ok(Data::LessonGet {
        id: lesson.id,
        slug: lesson.slug,
        title: lesson.title,
        ord: lesson.ord,
        status: status_str,
        skip: lesson.skip,
        sections,
        quizzes,
    })
}

/// List lessons.
pub fn list(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let mut lessons = Vec::new();
    for l in db::list_lessons(&conn)? {
        let st = status::lesson_status(&conn, &l.id)?.map(|s| s.as_str().to_string());
        lessons.push(LessonListItem {
            id: l.id,
            title: l.title,
            ord: l.ord,
            status: st.unwrap_or(l.status),
            skip: l.skip,
        });
    }
    Ok(Data::LessonList {
        lessons,
        errors: Vec::new(),
    })
}

/// Show a lesson's status + progress.
pub fn show(paths: &Paths, course_slug: &str, id: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, id)?;
    let status_str = status::lesson_status(&conn, id)?
        .map(|s| s.as_str().to_string())
        .unwrap_or(lesson.status);
    let c = db::lesson_show_counts(&conn, id)?;
    Ok(Data::LessonShow {
        id: lesson.id,
        title: lesson.title,
        status: status_str,
        skip: lesson.skip,
        progress: LessonProgress {
            sections: c.sections,
            practice: c.practice,
            quizzes: c.quizzes,
            passing: c.passing,
            total: c.total,
        },
    })
}

/// Update a lesson from a spec (requires `--force`); re-renders the notebook.
pub fn update(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    spec_json: &str,
    force: bool,
) -> Result<Data, CarpenterError> {
    if !force {
        return Err(CarpenterError::Conflict(format!(
            "update requires --force: lesson {id}"
        )));
    }
    let spec: LessonSpec = store::parse_spec(spec_json)?;
    validate_lesson_spec(&spec)?;
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, id)?;
    let now = time::now_iso();
    db::delete_lesson_content(&conn, id)?;
    insert_lesson_content(&conn, id, &spec)?;
    db::update_lesson_meta(&conn, id, &spec.title, &now)?;

    let dir = lesson_dir(paths, course_slug, &lesson.slug, lesson.ord);
    if dir.exists() {
        let nb = notebook::render_to_string(&conn, id)?;
        store::atomic_write(&dir.join("lesson.ipynb"), nb.as_bytes())?;
    }
    Ok(Data::LessonUpdate {
        id: id.into(),
        updated: LessonRow {
            id: id.into(),
            slug: lesson.slug,
            title: spec.title,
            ord: lesson.ord,
            status: lesson.status,
            skip: lesson.skip,
            created_at: lesson.created_at,
            updated_at: now,
        },
    })
}

/// Delete a lesson (requires `--force`).
pub fn delete(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    force: bool,
) -> Result<Data, CarpenterError> {
    if !force {
        return Err(CarpenterError::Conflict(format!(
            "delete requires --force: lesson {id}"
        )));
    }
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, id)?;
    db::delete_lesson(&conn, id)?;
    let dir = lesson_dir(paths, course_slug, &lesson.slug, lesson.ord);
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    Ok(Data::LessonDelete {
        id: id.into(),
        deleted: true,
    })
}

/// Sync a lesson's notebook against the DB (3-way stub preservation).
pub fn sync(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    force: bool,
) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, id)?;
    let dir = lesson_dir(paths, course_slug, &lesson.slug, lesson.ord);
    let nb_path = dir.join("lesson.ipynb");
    let old: Value = if nb_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&nb_path).map_err(store::io_to_store)?)
            .unwrap_or_else(|_| json!({"cells": []}))
    } else {
        json!({"cells": []})
    };
    let (new_nb, conflicts) = notebook::sync_notebook(&old, &conn, id, force)?;
    fs::create_dir_all(&dir).map_err(store::io_to_store)?;
    let pretty = serde_json::to_string_pretty(&new_nb)
        .map_err(|e| CarpenterError::StoreError(format!("notebook encode failed: {e}")))?;
    store::atomic_write(&nb_path, pretty.as_bytes())?;
    Ok(Data::LessonSync {
        id: id.into(),
        synced: true,
        conflicts,
    })
}

/// Execute a lesson's notebook end-to-end in the course venv.
///
/// Strict (default): the first errored cell ⇒ `ExecuteError`. `--allow-errors`:
/// runs every cell and returns the full error list + counts. Always executed
/// internally with `allow_errors=True` so all cell outputs are captured.
pub fn execute(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    timeout: u64,
    allow_errors: bool,
) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let lesson = db::get_lesson(&conn, id)?;
    let course_dir = paths.course(course_slug);
    if !course_dir.join(".venv").exists() {
        return Err(CarpenterError::StoreError(format!(
            "no course venv for {course_slug} — run `carpenter venv create` first"
        )));
    }
    let dir = lesson_dir(paths, course_slug, &lesson.slug, lesson.ord);
    // Run nbconvert from the lesson dir so the kernel cwd resolves `import helper`.
    let timeout_arg = format!("--ExecutePreprocessor.timeout={timeout}");
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

    let nb_path = dir.join("lesson.ipynb");
    let nb_text = std::fs::read_to_string(&nb_path).map_err(store::io_to_store)?;
    let nb: Value = serde_json::from_str(&nb_text)
        .map_err(|e| CarpenterError::StoreError(format!("notebook unreadable: {e}")))?;
    let errors: Vec<ExecError> = notebook::scan_errors(&nb)
        .into_iter()
        .map(|e| ExecError {
            index: e.index,
            ename: e.ename,
            evalue: e.evalue,
        })
        .collect();
    if !allow_errors {
        if let Some(first) = errors.first() {
            return Err(CarpenterError::ExecuteError {
                message: format!("cell {} errored: {}", first.index, first.ename),
                details: json!({"index": first.index, "ename": first.ename, "evalue": first.evalue}),
            });
        }
    }
    let total = nb["cells"]
        .as_array()
        .map(|c| {
            c.iter()
                .filter(|cell| cell.get("cell_type").and_then(|v| v.as_str()) == Some("code"))
                .count() as i64
        })
        .unwrap_or(0);
    Ok(Data::LessonExecute {
        id: id.into(),
        executed: true,
        cells: ExecuteCells {
            total,
            ran: total,
            errored: errors.len() as i64,
        },
        errors,
    })
}

fn checkable_tree(
    conn: &rusqlite::Connection,
    owner_type: &str,
    c: &db::CheckableDb,
) -> Result<CheckableTree, CarpenterError> {
    let mut cases = Vec::new();
    for cc in db::list_cases(conn, owner_type, &c.id)? {
        let args: Vec<Value> = serde_json::from_str(&cc.args).unwrap_or_default();
        let kwargs: Map<String, Value> = serde_json::from_str(&cc.kwargs).unwrap_or_default();
        let expected: Value = serde_json::from_str(&cc.expected).unwrap_or(Value::Null);
        cases.push(CaseTree {
            id: cc.id,
            args,
            kwargs,
            expected,
            compare: cc.compare,
            ord: cc.ord,
        });
    }
    Ok(CheckableTree {
        id: c.id.clone(),
        name: c.name.clone(),
        signature: c.signature.clone(),
        prompt: c.prompt.clone(),
        cases,
        skip: c.skip,
        pass_or_fail: c.pass_or_fail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    const SPEC: &str = r##"{
      "title": "Arrays 101", "slug": "arrays-101", "order": 1,
      "sections": [
        { "title": "Intro", "snippets": [{"kind":"markdown","content":"# hi"}],
          "practice": [{"name":"sum_array","signature":"def sum_array(arr):","prompt":"sum it","cases":[{"args":[[1,2]],"expected":3}]}] }
      ],
      "quizzes": [{"name":"max_value","signature":"def max_value(arr):","cases":[{"args":[[3,1,2]],"expected":3,"compare":"set"}]}]
    }"##;

    fn lesson_path(paths: &Paths, course: &str) -> std::path::PathBuf {
        paths.course(course).join("lessons").join("01-arrays-101")
    }

    #[test]
    fn create_ok_renders_notebook_and_helper() {
        let (paths, slug) = testutil::setup();
        let data = create(&paths, &slug, SPEC).expect("create");
        let Data::LessonCreate { counts, path, .. } = data else {
            panic!("LessonCreate");
        };
        assert_eq!(counts.cases, 2);
        let dir = std::path::Path::new(&path);
        assert!(dir.join("lesson.ipynb").exists());
        assert!(dir.join("helper.py").exists());
    }

    #[test]
    fn create_rejects_first_snippet_not_markdown() {
        let (paths, slug) = testutil::setup();
        let bad = r#"{"title":"x","sections":[{"title":"s","snippets":[{"kind":"code","content":"x"}]}],"quizzes":[]}"#;
        let err = create(&paths, &slug, bad).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
    }

    #[test]
    fn create_dedups_slug_on_collision() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let Data::LessonCreate { id, .. } = create(&paths, &slug, SPEC).expect("dup") else {
            panic!();
        };
        assert!(id.starts_with("arrays-101-"), "{id}");
    }

    #[test]
    fn get_returns_full_tree() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let data = get(&paths, &slug, "arrays-101").expect("get");
        let Data::LessonGet {
            sections, quizzes, ..
        } = data
        else {
            panic!();
        };
        assert_eq!(sections.len(), 1);
        assert_eq!(quizzes[0].cases[0].compare, "set");
    }

    #[test]
    fn list_ok() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let Data::LessonList { lessons, .. } = list(&paths, &slug).expect("list") else {
            panic!();
        };
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].status, "not_started");
    }

    #[test]
    fn show_ok() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let Data::LessonShow { progress, .. } = show(&paths, &slug, "arrays-101").expect("show")
        else {
            panic!();
        };
        assert_eq!(progress.sections, 1);
        assert_eq!(progress.total, 2); // 1 practice + 1 quiz
    }

    #[test]
    fn update_ok() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let Data::LessonUpdate { updated, .. } = update(
            &paths,
            &slug,
            "arrays-101",
            r#"{"title":"Arrays 202","sections":[{"title":"s","snippets":[{"kind":"markdown","content":"x"}]}],"quizzes":[]}"#,
            true,
        )
        .expect("update")
        else {
            panic!();
        };
        assert_eq!(updated.title, "Arrays 202");
    }

    #[test]
    fn update_without_force_conflicts() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let err = update(&paths, &slug, "arrays-101", SPEC, false).unwrap_err();
        assert!(matches!(err, CarpenterError::Conflict(_)));
    }

    #[test]
    fn delete_ok() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let _ = delete(&paths, &slug, "arrays-101", true).expect("delete");
        assert!(!lesson_path(&paths, &slug).exists());
    }

    #[test]
    fn delete_without_force_conflicts() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let err = delete(&paths, &slug, "arrays-101", false).unwrap_err();
        assert!(matches!(err, CarpenterError::Conflict(_)));
    }

    #[test]
    fn execute_requires_venv() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let err = execute(&paths, &slug, "arrays-101", 30, false).unwrap_err();
        assert!(matches!(err, CarpenterError::StoreError(_)));
    }

    #[test]
    #[ignore = "needs a course venv + nbconvert (run manually)"]
    fn execute_runs_strict_and_allow_errors() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        crate::commands::venv::create(&paths, &slug, None).expect("venv");
        let _ = execute(&paths, &slug, "arrays-101", 30, true).expect("execute");
    }

    #[test]
    fn sync_preserves_learner_edited_stub() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let nb_path = lesson_path(&paths, &slug).join("lesson.ipynb");
        // simulate a learner filling the practice stub
        let mut nb: Value =
            serde_json::from_str(&std::fs::read_to_string(&nb_path).unwrap()).unwrap();
        nb["cells"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .for_each(|c| {
                if c.get("metadata").and_then(|m| m.get("managed")) == Some(&json!("practice-stub"))
                {
                    c["source"] = json!("def sum_array(arr):\n    return sum(arr)\n");
                }
            });
        std::fs::write(&nb_path, nb.to_string()).unwrap();

        let Data::LessonSync { conflicts, .. } =
            sync(&paths, &slug, "arrays-101", false).expect("sync")
        else {
            panic!();
        };
        assert!(conflicts.is_empty(), "{conflicts:?}"); // db unchanged ⇒ preserved silently
        let after: Value =
            serde_json::from_str(&std::fs::read_to_string(&nb_path).unwrap()).unwrap();
        let preserved = after["cells"].as_array().unwrap().iter().any(|c| {
            c.get("metadata").and_then(|m| m.get("managed")) == Some(&json!("practice-stub"))
                && c["source"].as_str() == Some("def sum_array(arr):\n    return sum(arr)\n")
        });
        assert!(preserved, "learner fill was not preserved");
    }

    #[test]
    fn sync_conflicts_when_db_changed_under_learner_edit() {
        let (paths, slug) = testutil::setup();
        create(&paths, &slug, SPEC).unwrap();
        let nb_path = lesson_path(&paths, &slug).join("lesson.ipynb");
        // learner edits the stub
        let mut nb: Value =
            serde_json::from_str(&std::fs::read_to_string(&nb_path).unwrap()).unwrap();
        nb["cells"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .for_each(|c| {
                if c.get("metadata").and_then(|m| m.get("managed")) == Some(&json!("practice-stub"))
                {
                    c["source"] = json!("def sum_array(arr):\n    return sum(arr)\n");
                }
            });
        std::fs::write(&nb_path, nb.to_string()).unwrap();
        // agent changes the DB signature (scaffold changes) under the edited stub
        let conn = db::open_course(&paths, &slug).unwrap();
        conn.execute(
            "UPDATE practice SET signature='def sum_array(a, b):' WHERE id='p1'",
            [],
        )
        .unwrap();

        let Data::LessonSync { conflicts, .. } =
            sync(&paths, &slug, "arrays-101", false).expect("sync")
        else {
            panic!();
        };
        assert_eq!(conflicts.len(), 1, "{conflicts:?}");
        assert_eq!(conflicts[0].reason, "db_changed");
        assert_eq!(conflicts[0].id, "p1");
        // without --force the learner source is left intact
        let after: Value =
            serde_json::from_str(&std::fs::read_to_string(&nb_path).unwrap()).unwrap();
        let kept =
            after["cells"].as_array().unwrap().iter().any(|c| {
                c["source"].as_str() == Some("def sum_array(arr):\n    return sum(arr)\n")
            });
        assert!(kept, "learner source should be left intact without --force");
    }
}
