//! The build-time-generated howto manual, embedded at compile time.
//!
//! `src/howto.gen.md` is produced by `cargo xtask gen-howto`; never hand-edit.

/// The full generated manual text.
pub const MANUAL: &str = include_str!("howto.gen.md");
