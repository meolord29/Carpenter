//! Goal models: authored spec + serialized row/list shapes.

use serde::{Deserialize, Serialize};

/// Authored goal definition (`docs/specs/05-goal-spec.md`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoalSpec {
    /// The bullet goal text (required, non-empty).
    pub text: String,
    /// Lesson ids covering this goal (default empty).
    #[serde(default)]
    pub covered_by: Vec<String>,
}

/// A goals table row.
#[derive(Debug, Clone)]
pub struct GoalRow {
    /// `g1`, `g2`, …
    pub id: String,
    /// always `course`.
    pub scope: String,
    /// course slug.
    pub scope_id: String,
    /// the goal text.
    pub text: String,
    /// `pending` | `achieved` | `skipped`.
    pub status: String,
    /// covering lesson ids.
    pub covered_by: Vec<String>,
    /// `true` if status is pinned (skip derivation).
    pub override_flag: bool,
    /// created_at.
    pub created_at: String,
}

/// One element of `goal list`.
#[derive(Debug, Clone, Serialize)]
pub struct GoalListItem {
    /// id.
    pub id: String,
    /// goal text.
    pub text: String,
    /// effective status (pinned if `override`, else derived).
    pub status: String,
    /// derived status ignoring override.
    pub derived_status: String,
    /// covering lesson ids.
    pub covered_by: Vec<String>,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::{GoalListItem, GoalSpec};
    use crate::models::Data;

    /// A representative `GoalSpec`.
    pub fn spec() -> GoalSpec {
        GoalSpec {
            text: String::from("Implement a hash map from scratch"),
            covered_by: vec![String::from("hashing-101")],
        }
    }

    /// `(cmd, input, note, data)` rows for the `goal` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "add --spec -",
                "GoalSpec",
                "",
                Data::GoalAdd {
                    id: String::from("g1"),
                    text: String::from("Implement a hash map from scratch"),
                    covered_by: vec![String::from("hashing-101")],
                    status: String::from("pending"),
                },
            ),
            (
                "list",
                "—",
                "",
                Data::GoalList {
                    goals: vec![GoalListItem {
                        id: String::from("g1"),
                        text: String::from("…"),
                        status: String::from("pending"),
                        derived_status: String::from("pending"),
                        covered_by: vec![String::from("hashing-101")],
                    }],
                },
            ),
            (
                "update <id> [--status <S>] [--covered-by …]",
                "—",
                "`<S>` pins (`override=1`) or `derived` resumes",
                Data::GoalUpdate {
                    id: String::from("g1"),
                    status: String::from("achieved"),
                    override_field: true,
                    covered_by: vec![String::from("hashing-101")],
                },
            ),
            (
                "remove <id> --force",
                "—",
                "`Conflict` without `--force`",
                Data::GoalRemove {
                    id: String::from("g1"),
                    deleted: true,
                },
            ),
        ]
    }
}
