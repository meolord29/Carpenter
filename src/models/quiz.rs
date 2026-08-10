//! Output shapes for `quiz` commands.

use serde::Serialize;

use crate::models::execute::ExecError;

/// One per-case result (from `last_check`).
#[derive(Debug, Clone, Serialize)]
pub struct CaseResult {
    /// case id (`c1`…).
    pub case_id: String,
    /// whether it passed.
    pub passed: bool,
    /// error text (omitted if none).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// One element of `quiz list`.
#[derive(Debug, Clone, Serialize)]
pub struct QuizListItem {
    /// quiz id.
    pub id: String,
    /// owning lesson id.
    pub lesson_id: String,
    /// function name.
    pub name: String,
    /// number of cases.
    pub case_count: i64,
    /// skip flag.
    pub skip: bool,
    /// last-check pass flag.
    pub pass_or_fail: bool,
}

/// One quiz in `quiz run` output.
#[derive(Debug, Clone, Serialize)]
pub struct QuizRunItem {
    /// quiz id.
    pub quiz_id: String,
    /// skip flag.
    pub skipped: bool,
    /// last-check pass flag.
    pub pass_or_fail: bool,
    /// cases passed.
    pub passed: i64,
    /// total cases.
    pub total: i64,
    /// per-case results.
    pub cases: Vec<CaseResult>,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::*;
    use crate::models::Data;

    /// `(cmd, input, note, data)` rows for the `quiz` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        let _ = ExecError {
            index: 0,
            ename: String::new(),
            evalue: String::new(),
        };
        vec![
            (
                "run [lesson_id] [--timeout 30]",
                "—",
                "nbconvert in the course venv; `ExecuteError` on scaffolding errors; `StoreError` if no venv",
                Data::QuizRun {
                    lesson_id: String::from("arrays-101"),
                    quizzes: vec![QuizRunItem {
                        quiz_id: String::from("q1"),
                        skipped: false,
                        pass_or_fail: true,
                        passed: 1,
                        total: 1,
                        cases: vec![CaseResult {
                            case_id: String::from("c1"),
                            passed: true,
                            error: None,
                        }],
                    }],
                    saved: true,
                },
            ),
            (
                "list [lesson_id]",
                "—",
                "",
                Data::QuizList {
                    quizzes: vec![QuizListItem {
                        id: String::from("q1"),
                        lesson_id: String::from("arrays-101"),
                        name: String::from("max_value"),
                        case_count: 1,
                        skip: false,
                        pass_or_fail: false,
                    }],
                },
            ),
            (
                "show <quiz_id>",
                "—",
                "",
                Data::QuizShow {
                    id: String::from("q1"),
                    lesson_id: String::from("arrays-101"),
                    name: String::from("max_value"),
                    signature: String::from("def max_value(arr):"),
                    prompt: String::from("…"),
                    cases: 1,
                    skip: false,
                    pass_or_fail: false,
                },
            ),
            (
                "results <quiz_id>",
                "—",
                "live snapshot from `last_check` (no history)",
                Data::QuizResults {
                    quiz_id: String::from("q1"),
                    skipped: false,
                    pass_or_fail: true,
                    passed: 1,
                    total: 1,
                    cases: vec![CaseResult {
                        case_id: String::from("c1"),
                        passed: true,
                        error: None,
                    }],
                },
            ),
        ]
    }
}
