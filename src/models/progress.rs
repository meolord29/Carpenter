//! Output shapes for `progress` commands (docs/specs/13-progress.md).

use serde::Serialize;

/// One element of `progress show` (live per-lesson state).
#[derive(Debug, Clone, Serialize)]
pub struct ProgressLesson {
    /// lesson id (slug).
    pub id: String,
    /// lesson title.
    pub title: String,
    /// derived status.
    pub status: String,
    /// whole-lesson skip flag.
    pub skip: bool,
    /// non-skipped practice+quiz with `pass_or_fail=1`.
    pub passing: i64,
    /// non-skipped practice+quiz.
    pub total: i64,
}

/// Lesson roll-up in `progress summary`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LessonRollup {
    /// all lessons.
    pub total: i64,
    /// lessons derived `complete`.
    pub complete: i64,
    /// lessons derived `in_progress`.
    pub in_progress: i64,
    /// lessons derived `skipped`.
    pub skipped: i64,
}

/// Quiz roll-up in `progress summary` (non-skipped quizzes).
#[derive(Debug, Clone, Default, Serialize)]
pub struct QuizRollup {
    /// non-skipped quizzes with `pass_or_fail=1`.
    pub passing: i64,
    /// non-skipped quizzes.
    pub total: i64,
}

/// Goal roll-up in `progress summary`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GoalRollup {
    /// all goals.
    pub total: i64,
    /// goals whose effective status is `achieved` (override-aware).
    pub achieved: i64,
}

/// Per-kind note counts (`notes.by_kind`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct NotesByKind {
    /// gap notes.
    pub gap: i64,
    /// mistake notes.
    pub mistake: i64,
    /// strength notes.
    pub strength: i64,
    /// pattern notes.
    pub pattern: i64,
    /// progress notes.
    pub progress: i64,
}

/// Note roll-up in `progress summary`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NoteRollup {
    /// all notes.
    pub total: i64,
    /// notes with `status='open'`.
    pub open: i64,
    /// notes with `recurrence='recurring'`.
    pub recurring: i64,
    /// counts per kind (all notes).
    pub by_kind: NotesByKind,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::*;
    use crate::models::Data;

    /// `(cmd, input, note, data)` rows for the `progress` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "show",
                "—",
                "live per-lesson state (`passing`/`total` over non-skipped practice+quiz)",
                Data::ProgressShow {
                    lessons: vec![ProgressLesson {
                        id: String::from("arrays-101"),
                        title: String::from("Arrays 101"),
                        status: String::from("in_progress"),
                        skip: false,
                        passing: 1,
                        total: 2,
                    }],
                },
            ),
            (
                "summary",
                "—",
                "`notes.by_kind` is an object keyed by kind; no history (adr/010)",
                Data::ProgressSummary {
                    lessons: LessonRollup {
                        total: 1,
                        complete: 0,
                        in_progress: 1,
                        skipped: 0,
                    },
                    quizzes: QuizRollup {
                        passing: 1,
                        total: 1,
                    },
                    goals: GoalRollup {
                        total: 1,
                        achieved: 0,
                    },
                    notes: NoteRollup {
                        total: 1,
                        open: 1,
                        recurring: 0,
                        by_kind: NotesByKind {
                            gap: 1,
                            mistake: 0,
                            strength: 0,
                            pattern: 0,
                            progress: 0,
                        },
                    },
                },
            ),
        ]
    }
}
