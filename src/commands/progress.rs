//! `progress` commands — show (per-lesson live state) + summary (roll-up).

use crate::core::error::CarpenterError;
use crate::core::store::Paths;
use crate::core::{db, status};
use crate::models::progress::{
    GoalRollup, LessonRollup, NoteRollup, NotesByKind, ProgressLesson, QuizRollup,
};
use crate::models::Data;

/// Per-lesson live progress (`passing`/`total` over non-skipped practice+quiz).
pub fn show(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let mut lessons = Vec::new();
    for l in db::list_lessons(&conn)? {
        let (st, passing, total) = match status::lesson_status(&conn, &l.id)? {
            Some(s) => {
                let inp = db::lesson_status_inputs(&conn, &l.id)?
                    .map(|i| (i.passing_items, i.total_items))
                    .unwrap_or((0, 0));
                (s.as_str().to_string(), inp.0, inp.1)
            }
            None => (l.status.clone(), 0, 0),
        };
        lessons.push(ProgressLesson {
            id: l.id,
            title: l.title,
            status: st,
            skip: l.skip,
            passing,
            total,
        });
    }
    Ok(Data::ProgressShow { lessons })
}

/// Course roll-up: lessons (by status), quizzes, goals, notes (incl. `by_kind`).
pub fn summary(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    // lessons: derive each status.
    let mut roll = LessonRollup::default();
    for l in db::list_lessons(&conn)? {
        roll.total += 1;
        match status::lesson_status(&conn, &l.id)? {
            Some(status::LessonStatus::Complete) => roll.complete += 1,
            Some(status::LessonStatus::InProgress) => roll.in_progress += 1,
            Some(status::LessonStatus::Skipped) => roll.skipped += 1,
            Some(status::LessonStatus::NotStarted) | None => {}
        }
    }
    // quizzes: non-skipped roll-up.
    let (q_passing, q_total) = db::quiz_rollup(&conn)?;
    // goals: total + achieved (override-aware).
    let mut goals = GoalRollup::default();
    for g in db::list_goals(&conn)? {
        goals.total += 1;
        if status::goal_effective(&conn, &g)? == status::GoalStatus::Achieved {
            goals.achieved += 1;
        }
    }
    // notes: aggregate counts.
    let nc = db::note_counts(&conn)?;
    let notes = NoteRollup {
        total: nc.total,
        open: nc.open,
        recurring: nc.recurring,
        by_kind: NotesByKind {
            gap: nc.gap,
            mistake: nc.mistake,
            strength: nc.strength,
            pattern: nc.pattern,
            progress: nc.progress,
        },
    };
    Ok(Data::ProgressSummary {
        lessons: roll,
        quizzes: QuizRollup {
            passing: q_passing,
            total: q_total,
        },
        goals,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    const LESSON: &str = "title: Arrays\nslug: arrays\nsections:\n  - title: s\n    snippets:\n      - kind: markdown\n        content: hi\n    practice:\n      - name: f\n        signature: \"def f(x):\"\n        cases:\n          - args: [1]\n            expected: 1\nquizzes:\n  - name: max_value\n    signature: \"def max_value(arr):\"\n    cases:\n      - args:\n          - [3, 1, 2]\n        expected: 3\n";

    /// Set up one lesson with one practice + one quiz (total=2 non-skipped items).
    fn setup() -> (Paths, String) {
        let (paths, slug) = testutil::setup();
        crate::commands::lesson::create(&paths, &slug, LESSON).expect("create lesson");
        (paths, slug)
    }

    #[test]
    fn show_reports_live_per_lesson_state() {
        let (paths, slug) = setup();
        // mark the practice as passing via raw conn (helper is Python-only)
        let conn = db::open_course(&paths, &slug).unwrap();
        conn.execute("UPDATE practice SET pass_or_fail=1 WHERE id='p1'", [])
            .unwrap();
        drop(conn);
        let Data::ProgressShow { lessons } = show(&paths, &slug).expect("show") else {
            panic!();
        };
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].id, "arrays");
        assert_eq!(lessons[0].status, "in_progress");
        assert_eq!((lessons[0].passing, lessons[0].total), (1, 2));
    }

    #[test]
    fn summary_roll_up_math() {
        let (paths, slug) = setup();
        let conn = db::open_course(&paths, &slug).unwrap();
        conn.execute("UPDATE practice SET pass_or_fail=1 WHERE id='p1'", [])
            .unwrap();
        conn.execute("UPDATE quizzes SET pass_or_fail=1 WHERE id='q1'", [])
            .unwrap();
        drop(conn);
        crate::commands::notes::add(&paths, &slug, "kind: gap\ntags: [x]\ntext: a\n").unwrap();
        crate::commands::notes::add(&paths, &slug, "kind: strength\ntags: [x]\ntext: b\n").unwrap();

        let Data::ProgressSummary {
            lessons,
            quizzes,
            goals,
            notes,
        } = summary(&paths, &slug).expect("summary")
        else {
            panic!();
        };
        // one lesson, now complete
        assert_eq!(lessons.total, 1);
        assert_eq!(lessons.complete, 1);
        assert_eq!(lessons.in_progress, 0);
        assert_eq!(lessons.skipped, 0);
        // one quiz, passing
        assert_eq!((quizzes.passing, quizzes.total), (1, 1));
        // no goals
        assert_eq!((goals.total, goals.achieved), (0, 0));
        // two notes: 2 total, 2 open, by_kind gap=1 strength=1
        assert_eq!(notes.total, 2);
        assert_eq!(notes.open, 2);
        assert_eq!((notes.by_kind.gap, notes.by_kind.strength), (1, 1));
        assert_eq!(notes.by_kind.mistake, 0);
    }

    #[test]
    fn summary_counts_skipped_lesson() {
        let (paths, slug) = setup();
        crate::commands::skip::skip(&paths, &slug, "lesson", "arrays", false).expect("skip");
        let Data::ProgressSummary { lessons, .. } = summary(&paths, &slug).expect("summary") else {
            panic!();
        };
        assert_eq!((lessons.total, lessons.skipped), (1, 1));
    }
}
