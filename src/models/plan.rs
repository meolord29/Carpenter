//! Plan models: authored spec + serialized row/list shapes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Authored plan definition (`docs/specs/04-plan-spec.md`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanSpec {
    /// Plan title (required).
    pub title: String,
    /// Bullet goals; become `goals` rows on `confirm` (course scope).
    #[serde(default)]
    pub goals: Vec<String>,
    /// Maps each goal to covering lessons: keys are `goal_index_<i>` where `<i>`
    /// is the 0-based index into `goals[]` (range-checked at `create`).
    #[serde(default)]
    pub links: BTreeMap<String, Vec<String>>,
}

/// A plans table row.
#[derive(Debug, Clone, Serialize)]
pub struct PlanRow {
    /// `pl1`, `pl2`, …
    pub id: String,
    /// `course` | `lesson`.
    pub scope: String,
    /// course slug | lesson id.
    pub scope_id: String,
    /// title.
    pub title: String,
    /// stored body (JSON of `{goals, links}` for course scope).
    pub content: String,
    /// created_at.
    pub created_at: String,
    /// `None` until `plan confirm`.
    pub confirmed_at: Option<String>,
}

/// One element of `plan list`.
#[derive(Debug, Clone, Serialize)]
pub struct PlanListItem {
    /// id.
    pub id: String,
    /// scope.
    pub scope: String,
    /// scope_id.
    pub scope_id: String,
    /// title.
    pub title: String,
    /// whether the plan is confirmed.
    pub confirmed: bool,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::{PlanListItem, PlanRow, PlanSpec};
    use crate::models::Data;
    use std::collections::BTreeMap;

    /// A representative course-scope `PlanSpec`.
    pub fn spec() -> PlanSpec {
        let mut links = BTreeMap::new();
        links.insert(
            String::from("goal_index_0"),
            vec![String::from("arrays-101"), String::from("hashing-101")],
        );
        PlanSpec {
            title: String::from("Data Structures — course plan"),
            goals: vec![
                String::from("Know array/list internals"),
                String::from("Implement a hash map from scratch"),
            ],
            links,
        }
    }

    /// `(cmd, input, note, data)` rows for the `plan` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "create --scope course|lesson [--lesson <id>] --spec -",
                "PlanSpec",
                "draft; `links` indexes range-checked here; `--scope lesson` needs `--lesson <id>`",
                Data::PlanCreate {
                    id: String::from("pl1"),
                    scope: String::from("course"),
                    scope_id: String::from("<slug>"),
                    title: String::from("…"),
                    content: String::from("{goals, links}"),
                    confirmed: false,
                },
            ),
            (
                "show <id>",
                "—",
                "",
                Data::PlanShow {
                    id: String::from("pl1"),
                    scope: String::from("course"),
                    scope_id: String::from("<slug>"),
                    title: String::from("…"),
                    content: String::from("{goals, links}"),
                    confirmed_at: None,
                },
            ),
            (
                "list [--scope course|lesson]",
                "—",
                "",
                Data::PlanList {
                    plans: vec![PlanListItem {
                        id: String::from("pl1"),
                        scope: String::from("course"),
                        scope_id: String::from("<slug>"),
                        title: String::from("…"),
                        confirmed: false,
                    }],
                },
            ),
            (
                "confirm <id>",
                "—",
                "course scope creates `goals` rows",
                Data::PlanConfirm {
                    id: String::from("pl1"),
                    confirmed: true,
                    confirmed_at: String::from("2026-08-09T12:00:00Z"),
                    goals_created: vec![String::from("g1"), String::from("g2")],
                },
            ),
            (
                "update <id> --spec -",
                "PlanSpec",
                "`Conflict` if already confirmed",
                Data::PlanUpdate {
                    id: String::from("pl1"),
                    updated: PlanRow {
                        id: String::from("pl1"),
                        scope: String::from("course"),
                        scope_id: String::from("<slug>"),
                        title: String::from("…"),
                        content: String::from("{goals, links}"),
                        created_at: String::from("2026-08-09T12:00:00Z"),
                        confirmed_at: None,
                    },
                },
            ),
            (
                "delete <id> --force",
                "—",
                "`Conflict` if confirmed without `--force`",
                Data::PlanDelete {
                    id: String::from("pl1"),
                    deleted: true,
                },
            ),
        ]
    }
}
