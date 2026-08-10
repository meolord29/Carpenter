//! Output shapes for notebook execution (`lesson execute`, quiz-run errors).

use serde::Serialize;

/// One cell-execution error.
#[derive(Debug, Clone, Serialize)]
pub struct ExecError {
    /// cell index.
    pub index: usize,
    /// exception name.
    pub ename: String,
    /// exception value.
    pub evalue: String,
}

/// Cell counts for `lesson execute --allow-errors`.
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteCells {
    /// total code cells.
    pub total: i64,
    /// cells that ran.
    pub ran: i64,
    /// cells that errored.
    pub errored: i64,
}
