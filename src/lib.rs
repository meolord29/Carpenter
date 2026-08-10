//! carpenter — an agent-driven CLI that builds Python/Jupyter learning material.
//!
//! SQLite is the source of truth; notebooks render from it. No embedded LLM:
//! an external agent (opencode) is the tutor; carpenter is deterministic storage,
//! rendering, and execution.

#![deny(missing_docs)]

pub mod app;
pub mod commands;
pub mod core;
pub mod manual;
pub mod models;
