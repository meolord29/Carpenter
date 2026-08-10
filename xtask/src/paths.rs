//! Workspace path resolution, independent of the runtime working directory.
//!
//! Tests run with CWD = the crate under test (`xtask/`), so relative paths like
//! `src/commands` would miss. Everything is rooted at the workspace dir instead.

use std::path::{Path, PathBuf};

/// The workspace root (parent of this crate's manifest dir), stable across runs.
pub fn workspace_root() -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(".").to_path_buf())
}
