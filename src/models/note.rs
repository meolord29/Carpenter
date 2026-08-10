//! Note models: authored spec + serialized row shape (docs/specs/06, 14).

use serde::{Deserialize, Serialize};

/// Authored note definition (`docs/specs/06-note-spec.md`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoteSpec {
    /// `gap|mistake|strength|pattern|progress` (required).
    pub kind: String,
    /// Free-form tags (default empty).
    #[serde(default)]
    pub tags: Vec<String>,
    /// `new` (default) | `recurring` — authored; never auto-changed.
    #[serde(default = "default_recurrence")]
    pub recurrence: String,
    /// A lesson/quiz id, stored as free text (no FK).
    #[serde(default)]
    pub related: Option<String>,
    /// The note body (required, non-empty).
    pub text: String,
}

fn default_recurrence() -> String {
    String::from("new")
}

/// A note as surfaced in command output (`docs/specs/14-notes.md`).
#[derive(Debug, Clone, Serialize)]
pub struct NoteItem {
    /// `n1`, `n2`, …
    pub id: String,
    /// `gap|mistake|strength|pattern|progress`.
    pub kind: String,
    /// tags.
    pub tags: Vec<String>,
    /// `open` | `resolved`.
    pub status: String,
    /// `new` | `recurring` (author-owned).
    pub recurrence: String,
    /// free lesson/quiz ref (may be empty).
    pub related: String,
    /// the note body.
    pub text: String,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::{NoteItem, NoteSpec};
    use crate::models::Data;

    /// A representative `NoteSpec`.
    pub fn spec() -> NoteSpec {
        NoteSpec {
            kind: String::from("gap"),
            tags: vec![String::from("recursion")],
            recurrence: String::from("new"),
            related: Some(String::from("q2")),
            text: String::from("Learner struggles with base cases."),
        }
    }

    fn item() -> NoteItem {
        NoteItem {
            id: String::from("n1"),
            kind: String::from("gap"),
            tags: vec![String::from("recursion")],
            status: String::from("open"),
            recurrence: String::from("new"),
            related: String::from("q2"),
            text: String::from("Learner struggles with base cases."),
        }
    }

    /// `(cmd, input, note, data)` rows for the `notes` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "add --spec -",
                "NoteSpec",
                "`related_open` = open notes sharing ≥1 tag (excluding self) — advisory; `recurrence` is never auto-changed",
                Data::NotesAdd {
                    id: String::from("n1"),
                    kind: String::from("gap"),
                    tags: vec![String::from("recursion")],
                    status: String::from("open"),
                    recurrence: String::from("new"),
                    related: String::from("q2"),
                    text: String::from("Learner struggles with base cases."),
                    related_open: vec![],
                },
            ),
            (
                "show <id>",
                "—",
                "`NotFound` if absent",
                Data::NotesShow {
                    notes: vec![item()],
                },
            ),
            (
                "list",
                "—",
                "corrupt rows surface in `errors[]`",
                Data::NotesList {
                    notes: vec![item()],
                    errors: vec![],
                },
            ),
            (
                "update <id> --spec -",
                "NoteSpec",
                "",
                Data::NotesUpdate {
                    id: String::from("n1"),
                    updated: item(),
                },
            ),
            (
                "resolve <id>",
                "—",
                "",
                Data::NotesResolve {
                    id: String::from("n1"),
                    status: String::from("resolved"),
                },
            ),
            (
                "remove <id> --force",
                "—",
                "`Conflict` without `--force`",
                Data::NotesRemove {
                    id: String::from("n1"),
                    deleted: true,
                },
            ),
        ]
    }
}
