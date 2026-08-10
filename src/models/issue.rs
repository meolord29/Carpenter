//! Bug/feature issue models: authored spec + list item (docs/specs/07, 15).

use serde::{Deserialize, Serialize};

/// Authored bug *or* feature definition (`docs/specs/07-bug-feature-spec.md`).
///
/// `repro` is bug-only, `rationale` is feature-only; the command rejects using
/// the wrong one (or mixing both).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IssueSpec {
    /// Title (required, non-empty).
    pub title: String,
    /// Description (required, non-empty).
    pub description: String,
    /// Repro steps — bug only.
    #[serde(default)]
    pub repro: Option<String>,
    /// Rationale — feature only.
    #[serde(default)]
    pub rationale: Option<String>,
}

/// One issue in a `list` (`docs/specs/15-bug-feature.md`).
#[derive(Debug, Clone, Serialize)]
pub struct IssueListItem {
    /// id (`b1`/`f1`…).
    pub id: String,
    /// title.
    pub title: String,
    /// `open` | `resolved`.
    pub status: String,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::{IssueListItem, IssueSpec};
    use crate::models::Data;

    /// A representative bug `IssueSpec`.
    pub fn spec() -> IssueSpec {
        IssueSpec {
            title: String::from("quiz run ignores --timeout"),
            description: String::from("The timeout flag has no effect."),
            repro: Some(String::from("carpenter quiz run 01 …")),
            rationale: None,
        }
    }

    /// `(cmd, input, note, data)` rows for the bug/feature output contract
    /// (identical shape; `bug` writes `bug/`, `feature` writes `feature_request/`).
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "file --spec -",
                "IssueSpec",
                "`id` is `<prefix><N>` (`b1`… for bug; `f1`… for feature), `max+1` per kind",
                Data::IssueFile {
                    id: String::from("b1"),
                    path: String::from("~/.config/carpenter/bug/b1.json"),
                    status: String::from("open"),
                },
            ),
            (
                "list",
                "—",
                "corrupt files surface in `errors[]`",
                Data::IssueList {
                    items: vec![IssueListItem {
                        id: String::from("b1"),
                        title: String::from("quiz run ignores --timeout"),
                        status: String::from("open"),
                    }],
                    errors: vec![],
                },
            ),
            (
                "show <id>",
                "—",
                "`NotFound` if absent",
                Data::IssueShow {
                    id: String::from("b1"),
                    title: String::from("quiz run ignores --timeout"),
                    description: String::from("The timeout flag has no effect."),
                    repro: Some(String::from("carpenter quiz run 01 …")),
                    rationale: None,
                    status: String::from("open"),
                    resolved_ts: None,
                },
            ),
            (
                "resolve <id>",
                "—",
                "",
                Data::IssueResolve {
                    id: String::from("b1"),
                    status: String::from("resolved"),
                    resolved_ts: String::from("2026-08-09T12:00:00Z"),
                },
            ),
        ]
    }
}
