//! `skip` command — DB-authored skip flags (adr/011).

use crate::core::error::CarpenterError;
use crate::core::{db, status, store::Paths};
use crate::models::Data;

/// Set (or with `--off`, clear) the skip flag on a lesson, quiz, or practice
/// item. Skipped items are excluded from lesson status derivation, and
/// `lessons.skip=1` forces lesson status `skipped`. The change does not
/// re-execute the notebook; the rendered `managed=skip-config` cell reads the
/// DB and reflects the new state on the next `lesson sync`.
pub fn skip(
    paths: &Paths,
    course_slug: &str,
    scope: &str,
    id: &str,
    off: bool,
) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let (table, lesson_id) = match scope {
        "lesson" => {
            db::get_lesson(&conn, id)?;
            ("lessons", id.to_string())
        }
        "quiz" => {
            db::get_quiz(&conn, id)?;
            ("quizzes", db::quiz_lesson_id(&conn, id)?)
        }
        "practice" => {
            db::get_practice(&conn, id)?;
            ("practice", db::practice_lesson_id(&conn, id)?)
        }
        other => {
            return Err(CarpenterError::ValidationError(format!(
                "invalid --scope {other:?} (lesson|quiz|practice)"
            )))
        }
    };
    let skip = !off;
    db::set_skip(&conn, table, id, skip)?;
    if let Some(st) = status::lesson_status(&conn, &lesson_id)? {
        db::set_lesson_status(&conn, &lesson_id, st.as_str())?;
    }
    Ok(Data::Skip {
        scope: scope.into(),
        id: id.into(),
        skip,
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

    fn lesson_status_of(paths: &Paths, slug: &str) -> String {
        match crate::commands::lesson::show(paths, slug, "arrays").expect("show") {
            Data::LessonShow { status, .. } => status,
            _ => unreachable!(),
        }
    }

    #[test]
    fn skip_sets_and_clears_per_scope() {
        let (paths, slug) = setup_lesson();
        for (scope, id) in [("lesson", "arrays"), ("quiz", "q1"), ("practice", "p1")] {
            let Data::Skip { skip: flag, .. } =
                skip(&paths, &slug, scope, id, false).expect("skip")
            else {
                panic!();
            };
            assert!(flag);
            let Data::Skip { skip: flag, .. } = skip(&paths, &slug, scope, id, true).expect("off")
            else {
                panic!();
            };
            assert!(!flag);
        }
    }

    #[test]
    fn skip_not_found_on_bad_id() {
        let (paths, slug) = setup_lesson();
        for scope in ["lesson", "quiz", "practice"] {
            let err = skip(&paths, &slug, scope, "nope", false).unwrap_err();
            assert!(matches!(err, CarpenterError::NotFound(_)), "{scope}: {err}");
        }
    }

    #[test]
    fn skip_invalid_scope_is_validation_error() {
        let (paths, slug) = setup_lesson();
        let err = skip(&paths, &slug, "section", "s1", false).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
    }

    #[test]
    fn skip_excludes_item_from_derivation() {
        let (paths, slug) = setup_lesson();
        let conn = db::open_course(&paths, &slug).expect("open");
        conn.execute("UPDATE practice SET pass_or_fail=1 WHERE id='p1'", [])
            .expect("pass p1");
        drop(conn);
        assert_eq!(lesson_status_of(&paths, &slug), "in_progress");
        skip(&paths, &slug, "quiz", "q1", false).expect("skip q1");
        assert_eq!(lesson_status_of(&paths, &slug), "complete");
        skip(&paths, &slug, "quiz", "q1", true).expect("unskip q1");
        assert_eq!(lesson_status_of(&paths, &slug), "in_progress");
    }

    #[test]
    fn skip_lesson_forces_skipped_status() {
        let (paths, slug) = setup_lesson();
        skip(&paths, &slug, "lesson", "arrays", false).expect("skip");
        assert_eq!(lesson_status_of(&paths, &slug), "skipped");
        skip(&paths, &slug, "lesson", "arrays", true).expect("off");
        assert_eq!(lesson_status_of(&paths, &slug), "not_started");
    }
}
