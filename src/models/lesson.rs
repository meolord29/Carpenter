//! Lesson models: authored spec + serialized counts/tree shapes.
//!
//! `Checkable` is a *shape* shared by practice items and quizzes (composition);
//! they live in separate tables with different parents (no shared abstraction).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::core::compare::CompareMode;

// ---- authored spec (input) ----

/// Authored lesson definition (`docs/specs/03-lesson-spec.md`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LessonSpec {
    /// Lesson title (required).
    pub title: String,
    /// Slug; derived from title if absent.
    #[serde(default)]
    pub slug: Option<String>,
    /// Order; appended (max+1) if absent.
    #[serde(default)]
    pub order: Option<i64>,
    /// Sections (teaching + practice).
    pub sections: Vec<SectionSpec>,
    /// End-of-notebook quizzes.
    #[serde(default)]
    pub quizzes: Vec<CheckableSpec>,
}

/// A section of a lesson.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SectionSpec {
    /// Section title.
    pub title: String,
    /// Snippets; `snippets[0].kind == "markdown"` (enforced).
    pub snippets: Vec<SnippetSpec>,
    /// Practice items under this section.
    #[serde(default)]
    pub practice: Vec<CheckableSpec>,
}

/// One snippet (renders as one cell).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnippetSpec {
    /// `markdown` | `code`.
    pub kind: String,
    /// Cell source.
    pub content: String,
}

/// A practice item or quiz (the shared `Checkable` shape).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckableSpec {
    /// Function name.
    pub name: String,
    /// e.g. `def sum_array(arr):`.
    pub signature: String,
    /// Optional prompt shown in the stub.
    #[serde(default)]
    pub prompt: String,
    /// Test cases (array index ⇒ `ord`).
    #[serde(default)]
    pub cases: Vec<CaseSpec>,
    /// Optional **author reference solution** — Python source that defines the
    /// fn named `name`. Author-only: never rendered into the notebook, never
    /// shown to the learner. Used by `lesson verify` to self-check the answer
    /// key ([adr/015](../../docs/adr/015-reference-solution-verify.md)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
}

/// One test case.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaseSpec {
    /// Compare mode (default `exact`).
    #[serde(default)]
    pub compare: CompareMode,
    /// Positional args (default `[]`).
    #[serde(default)]
    pub args: Vec<Value>,
    /// Keyword args (default `{}`).
    #[serde(default)]
    pub kwargs: Map<String, Value>,
    /// Expected value (required; never shown to learner).
    pub expected: Value,
}

// ---- output shapes ----

/// Counts returned by `lesson create`.
#[derive(Debug, Clone, Serialize)]
pub struct LessonCounts {
    /// sections.
    pub sections: i64,
    /// practice items.
    pub practice: i64,
    /// quizzes.
    pub quizzes: i64,
    /// test cases.
    pub cases: i64,
}

/// A snippet as output in the get-tree.
#[derive(Debug, Clone, Serialize)]
pub struct SnippetOut {
    /// id (e.g. `sn1`).
    pub id: String,
    /// `markdown` | `code`.
    pub kind: String,
    /// cell source.
    pub content: String,
}

/// A practice/quiz node in the get-tree.
#[derive(Debug, Clone, Serialize)]
pub struct CheckableTree {
    /// id.
    pub id: String,
    /// function name.
    pub name: String,
    /// signature.
    pub signature: String,
    /// prompt.
    pub prompt: String,
    /// test cases.
    pub cases: Vec<CaseTree>,
    /// skip flag.
    pub skip: bool,
    /// last-check pass flag.
    pub pass_or_fail: bool,
}

/// A test case in the get-tree.
#[derive(Debug, Clone, Serialize)]
pub struct CaseTree {
    /// id (`c1`…).
    pub id: String,
    /// positional args.
    pub args: Vec<Value>,
    /// keyword args.
    pub kwargs: Map<String, Value>,
    /// expected value.
    pub expected: Value,
    /// compare mode.
    pub compare: String,
    /// order.
    pub ord: i64,
}

/// A section in the get-tree.
#[derive(Debug, Clone, Serialize)]
pub struct SectionTree {
    /// id (`s1`…).
    pub id: String,
    /// title.
    pub title: String,
    /// snippets.
    pub snippets: Vec<SnippetOut>,
    /// order.
    pub ord: i64,
    /// practice items.
    pub practice: Vec<CheckableTree>,
}

// ---- lifecycle output shapes ----

/// A full lessons row (echoed as `updated:` on update).
#[derive(Debug, Clone, Serialize)]
pub struct LessonRow {
    /// id (slug).
    pub id: String,
    /// slug.
    pub slug: String,
    /// title.
    pub title: String,
    /// order.
    pub ord: i64,
    /// status.
    pub status: String,
    /// skip flag.
    pub skip: bool,
    /// created_at.
    pub created_at: String,
    /// updated_at.
    pub updated_at: String,
}

/// One element of `lesson list`.
#[derive(Debug, Clone, Serialize)]
pub struct LessonListItem {
    /// id.
    pub id: String,
    /// title.
    pub title: String,
    /// order.
    pub ord: i64,
    /// status.
    pub status: String,
    /// skip flag.
    pub skip: bool,
}

/// `lesson show` progress counts.
#[derive(Debug, Clone, Serialize)]
pub struct LessonProgress {
    /// section count.
    pub sections: i64,
    /// practice count.
    pub practice: i64,
    /// quiz count.
    pub quizzes: i64,
    /// non-skipped practice+quiz with `pass_or_fail=1`.
    pub passing: i64,
    /// non-skipped practice+quiz total.
    pub total: i64,
}

/// A `lesson sync` conflict entry.
#[derive(Debug, Clone, Serialize)]
pub struct LessonConflict {
    /// practice or quiz id.
    pub id: String,
    /// `learner_edited` | `db_changed`.
    pub reason: String,
}

/// One verified checkable (practice/quiz) in `lesson verify`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCheckable {
    /// `practice` | `quiz`.
    pub owner_type: String,
    /// owner id (`p1`/`q1`… in `<id>` mode; the fn `name` in `--spec` mode).
    pub owner_id: String,
    /// function name.
    pub name: String,
    /// whether a reference `solution` was present to verify against.
    pub has_solution: bool,
    /// cases that passed.
    pub passed: i64,
    /// total cases.
    pub total: i64,
    /// per-case results.
    pub cases: Vec<VerifyCase>,
}

/// One case result in `lesson verify`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCase {
    /// case id.
    pub case_id: String,
    /// pass flag.
    pub passed: bool,
    /// `repr(actual)` — present only on a comparison mismatch (never `expected`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub actual: Option<String>,
    /// exception/timeout text — present only on a failure (never `expected`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::*;
    use crate::models::Data;

    /// A representative `LessonSpec` (the `Arrays 101` example from the spec).
    pub fn spec() -> LessonSpec {
        LessonSpec {
            title: String::from("Arrays 101"),
            slug: Some(String::from("arrays-101")),
            order: Some(1),
            sections: vec![SectionSpec {
                title: String::from("What is an array"),
                snippets: vec![
                    SnippetSpec {
                        kind: String::from("markdown"),
                        content: String::from("An array stores items contiguously…"),
                    },
                    SnippetSpec {
                        kind: String::from("code"),
                        content: String::from("import numpy as np\nnp.array([1, 2, 3])"),
                    },
                ],
                practice: vec![CheckableSpec {
                    name: String::from("sum_array"),
                    signature: String::from("def sum_array(arr):"),
                    prompt: String::from("Return the sum of the array."),
                    solution: Some(String::from("def sum_array(arr):\n    return sum(arr)\n")),
                    cases: vec![
                        CaseSpec {
                            compare: CompareMode::Exact,
                            args: vec![Value::Array(vec![
                                Value::from(1),
                                Value::from(2),
                                Value::from(3),
                            ])],
                            kwargs: Map::new(),
                            expected: Value::from(6),
                        },
                        CaseSpec {
                            compare: CompareMode::Exact,
                            args: vec![Value::Array(vec![])],
                            kwargs: Map::new(),
                            expected: Value::from(0),
                        },
                    ],
                }],
            }],
            quizzes: vec![CheckableSpec {
                name: String::from("max_value"),
                signature: String::from("def max_value(arr):"),
                prompt: String::from("…"),
                solution: None,
                cases: vec![CaseSpec {
                    compare: CompareMode::Exact,
                    args: vec![Value::Array(vec![
                        Value::from(3),
                        Value::from(1),
                        Value::from(2),
                    ])],
                    kwargs: Map::new(),
                    expected: Value::from(3),
                }],
            }],
        }
    }

    /// `(cmd, input, note, data)` rows for the `lesson` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "create --spec -",
                "LessonSpec",
                "renders notebook + helper; `AlreadyExists` on duplicate slug",
                Data::LessonCreate {
                    id: String::from("arrays-101"),
                    slug: String::from("arrays-101"),
                    path: String::from("<root>/courses/<slug>/lessons/01-arrays-101"),
                    counts: LessonCounts {
                        sections: 1,
                        practice: 1,
                        quizzes: 1,
                        cases: 3,
                    },
                },
            ),
            (
                "get <id>",
                "—",
                "full tree (sections → practice → cases; quizzes → cases)",
                Data::LessonGet {
                    id: String::from("arrays-101"),
                    slug: String::from("arrays-101"),
                    title: String::from("Arrays 101"),
                    ord: 1,
                    status: String::from("not_started"),
                    skip: false,
                    sections: Vec::new(),
                    quizzes: Vec::new(),
                },
            ),
            (
                "list",
                "—",
                "",
                Data::LessonList {
                    lessons: vec![crate::models::LessonListItem {
                        id: String::from("arrays-101"),
                        title: String::from("Arrays 101"),
                        ord: 1,
                        status: String::from("not_started"),
                        skip: false,
                    }],
                    errors: Vec::new(),
                },
            ),
            (
                "show <id>",
                "—",
                "live `passing`/`total` (non-skipped)",
                Data::LessonShow {
                    id: String::from("arrays-101"),
                    title: String::from("Arrays 101"),
                    status: String::from("not_started"),
                    skip: false,
                    progress: crate::models::LessonProgress {
                        sections: 1,
                        practice: 1,
                        quizzes: 1,
                        passing: 0,
                        total: 2,
                    },
                },
            ),
            (
                "update <id> --spec - --force",
                "LessonSpec",
                "`Conflict` without `--force`; re-renders notebook",
                Data::LessonUpdate {
                    id: String::from("arrays-101"),
                    updated: crate::models::LessonRow {
                        id: String::from("arrays-101"),
                        slug: String::from("arrays-101"),
                        title: String::from("…"),
                        ord: 1,
                        status: String::from("not_started"),
                        skip: false,
                        created_at: String::from("2026-08-09T12:00:00Z"),
                        updated_at: String::from("2026-08-09T12:00:00Z"),
                    },
                },
            ),
            (
                "delete <id> --force",
                "—",
                "`Conflict` without `--force`",
                Data::LessonDelete {
                    id: String::from("arrays-101"),
                    deleted: true,
                },
            ),
            (
                "sync <id> [--force]",
                "—",
                "`conflicts[].reason` ∈ `learner_edited`\\|`db_changed`",
                Data::LessonSync {
                    id: String::from("arrays-101"),
                    synced: true,
                    conflicts: vec![crate::models::LessonConflict {
                        id: String::from("p1"),
                        reason: String::from("db_changed"),
                    }],
                },
            ),
            (
                "execute <id> [--allow-errors]",
                "—",
                "strict (default) ⇒ `ExecuteError` on first scaffolding error; `--allow-errors` lists `errors[]`",
                Data::LessonExecute {
                    id: String::from("arrays-101"),
                    executed: true,
                    cells: crate::models::execute::ExecuteCells {
                        total: 3,
                        ran: 3,
                        errored: 0,
                    },
                    errors: Vec::new(),
                },
            ),
            (
                "verify (<id> | --spec -) [--timeout <SECS>]",
                "LessonSpec (--spec) | — (<id>)",
                "runs each author `solution` vs its own cases; `--spec` is the pre-create key-lock, `<id>` re-verifies stored solutions",
                Data::LessonVerify {
                    lesson_id: Some(String::from("arrays-101")),
                    checked: 1,
                    passing: 1,
                    failing: 0,
                    checkables: vec![VerifyCheckable {
                        owner_type: String::from("practice"),
                        owner_id: String::from("p1"),
                        name: String::from("sum_array"),
                        has_solution: true,
                        passed: 1,
                        total: 1,
                        cases: vec![VerifyCase {
                            case_id: String::from("c1"),
                            passed: true,
                            actual: None,
                            error: None,
                        }],
                    }],
                },
            ),
            (
                "new [--out <FILE>]",
                "—",
                "emits a YAML lesson-spec template (block scalars + `solution`); stdout, or `--out` to write",
                Data::LessonNew {
                    yaml: Some(String::from("title: …\nsections: []\n")),
                    written_to: None,
                },
            ),
        ]
    }
}
