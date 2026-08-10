//! Status derivation — pure functions over DB rows.
//!
//! The single owner of lesson + goal status semantics
//! (`docs/data-model/04-status-derivation.md`). Lesson status derives from
//! `pass_or_fail` + `skip` (set by the helper / `skip`); goal status derives
//! from the completion of its `covered_by` lessons, unless pinned (`override=1`).

use rusqlite::Connection;

use crate::core::db;
use crate::core::error::CarpenterError;
use crate::models::GoalRow;

/// A lesson's derived status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LessonStatus {
    /// no non-skipped items, or none passing.
    NotStarted,
    /// some (not all) non-skipped items pass.
    InProgress,
    /// all non-skipped items pass.
    Complete,
    /// `lessons.skip = 1`.
    Skipped,
}

impl LessonStatus {
    /// The stored-status string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::Complete => "complete",
            Self::Skipped => "skipped",
        }
    }
}

/// A goal's status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalStatus {
    /// not all covering lessons complete (or none cover it).
    Pending,
    /// all `covered_by` lessons are complete.
    Achieved,
    /// pinned via `override=1`.
    Skipped,
}

impl GoalStatus {
    /// The status string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Achieved => "achieved",
            Self::Skipped => "skipped",
        }
    }
}

fn derive_lesson(inp: db::LessonStatusInputs) -> LessonStatus {
    if inp.skip {
        return LessonStatus::Skipped;
    }
    if inp.total_items == 0 {
        return LessonStatus::NotStarted;
    }
    if inp.passing_items == inp.total_items {
        return LessonStatus::Complete;
    }
    if inp.passing_items > 0 {
        return LessonStatus::InProgress;
    }
    LessonStatus::NotStarted
}

/// Derive a lesson's status; `None` if the lesson does not exist.
pub fn lesson_status(
    conn: &Connection,
    lesson_id: &str,
) -> Result<Option<LessonStatus>, CarpenterError> {
    Ok(db::lesson_status_inputs(conn, lesson_id)?.map(derive_lesson))
}

/// Is the lesson complete? False if absent (so a missing covering lesson keeps a
/// goal `pending`).
pub fn lesson_is_complete(conn: &Connection, lesson_id: &str) -> Result<bool, CarpenterError> {
    Ok(matches!(
        lesson_status(conn, lesson_id)?,
        Some(LessonStatus::Complete)
    ))
}

/// Derived goal status over a `covered_by` list, **ignoring** the override pin.
pub fn goal_derived_from(
    conn: &Connection,
    covered_by: &[String],
) -> Result<GoalStatus, CarpenterError> {
    if covered_by.is_empty() {
        return Ok(GoalStatus::Pending);
    }
    for lid in covered_by {
        if !lesson_is_complete(conn, lid)? {
            return Ok(GoalStatus::Pending);
        }
    }
    Ok(GoalStatus::Achieved)
}

/// Derived goal status, **ignoring** the override pin.
pub fn goal_derived(conn: &Connection, goal: &GoalRow) -> Result<GoalStatus, CarpenterError> {
    goal_derived_from(conn, &goal.covered_by)
}

/// Effective goal status: the pinned value if `override=1`, else derived.
pub fn goal_effective(conn: &Connection, goal: &GoalRow) -> Result<GoalStatus, CarpenterError> {
    if goal.override_flag {
        return Ok(match goal.status.as_str() {
            "achieved" => GoalStatus::Achieved,
            "skipped" => GoalStatus::Skipped,
            _ => GoalStatus::Pending,
        });
    }
    goal_derived(conn, goal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_status_strings() {
        assert_eq!(LessonStatus::NotStarted.as_str(), "not_started");
        assert_eq!(LessonStatus::Complete.as_str(), "complete");
        assert_eq!(GoalStatus::Pending.as_str(), "pending");
        assert_eq!(GoalStatus::Achieved.as_str(), "achieved");
    }

    #[test]
    fn derive_lesson_rules() {
        let inp = |skip, total, passing| db::LessonStatusInputs {
            skip,
            total_items: total,
            passing_items: passing,
        };
        assert_eq!(derive_lesson(inp(true, 0, 0)), LessonStatus::Skipped);
        assert_eq!(derive_lesson(inp(false, 0, 0)), LessonStatus::NotStarted);
        assert_eq!(derive_lesson(inp(false, 3, 3)), LessonStatus::Complete);
        assert_eq!(derive_lesson(inp(false, 3, 1)), LessonStatus::InProgress);
        assert_eq!(derive_lesson(inp(false, 3, 0)), LessonStatus::NotStarted);
    }
}
