//! Subprocess execution (uv + nbconvert) — the runtime isolation boundary.
//!
//! Learner Python never reaches carpenter's process; it runs inside nbconvert's
//! kernel via `uv run`. `uv` presence is checked up front (a missing `uv` is a
//! clear `StoreError`, not a cryptic spawn failure).

use std::path::Path;
use std::process::{Command, Output};

use crate::core::error::CarpenterError;

/// Is `uv` reachable on PATH?
pub fn uv_available() -> bool {
    Command::new("uv")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Map "uv missing" to a clear [`CarpenterError::StoreError`]. `found` lets the
/// error path be unit-tested without manipulating PATH.
pub fn require_uv(found: bool) -> Result<(), CarpenterError> {
    if found {
        Ok(())
    } else {
        Err(CarpenterError::StoreError(
            "`uv` is not on PATH — install uv (https://docs.astral.sh/uv/) \
             before using venv/execute/quiz commands."
                .into(),
        ))
    }
}

/// Run `uv` with args in `cwd`, returning its captured output.
pub fn run_uv(args: &[&str], cwd: &Path) -> Result<Output, CarpenterError> {
    Command::new("uv")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| CarpenterError::StoreError(format!("failed to run `uv`: {e}")))
}

/// Require `uv` present; run it; map a non-zero exit to `StoreError` w/ stderr.
pub fn run_uv_or_store(args: &[&str], cwd: &Path) -> Result<Output, CarpenterError> {
    require_uv(uv_available())?;
    let out = run_uv(args, cwd)?;
    if !out.status.success() {
        return Err(CarpenterError::StoreError(format!(
            "`uv {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out)
}

/// Run `cargo` with args in `cwd`, capturing output.
pub fn run_cargo(args: &[&str], cwd: &Path) -> Result<Output, CarpenterError> {
    Command::new("cargo")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| CarpenterError::StoreError(format!("failed to run `cargo`: {e}")))
}

/// Run `cargo` in `cwd`; map a non-zero exit to `StoreError` with trimmed stderr.
pub fn run_cargo_or_store(args: &[&str], cwd: &Path) -> Result<Output, CarpenterError> {
    let out = run_cargo(args, cwd)?;
    if !out.status.success() {
        return Err(CarpenterError::StoreError(format!(
            "`cargo {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_uv_missing_is_clear_store_error() {
        let err = require_uv(false).unwrap_err();
        assert!(matches!(err, CarpenterError::StoreError(_)));
        let (json, _) = crate::core::output::render(Err(err));
        assert!(json.contains("uv"), "{json}");
    }

    #[test]
    fn uv_is_available_in_this_env() {
        // this machine has uv installed; documents the assumption.
        assert!(uv_available());
    }
}
