//! Reusable envelope shapes shared across command outputs.

use serde::Serialize;

/// A corrupt-row entry surfaced in aggregate `list`/`show` `errors[]`.
#[derive(Debug, Clone, Serialize)]
pub struct RowError {
    /// The id of the offending row, if recoverable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Why the row could not be surfaced (e.g. `"corrupt_course"`).
    pub reason: String,
}
