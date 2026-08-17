//! Subprocess execution (uv + nbconvert) — the runtime isolation boundary.
//!
//! Learner Python never reaches carpenter's process; it runs inside nbconvert's
//! kernel via `uv run`. `uv` presence is checked up front (a missing `uv` is a
//! clear `StoreError`, not a cryptic spawn failure).

use std::path::Path;
use std::process::{Command, Output};

use fs4::fs_std::FileExt;

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

/// An acquired per-course notebook-execution lock (flock on `.exec.lock` in the
/// course dir). Released on drop — and on process death, so a crashed run never
/// leaves a stale lock.
pub struct ExecLock {
    file: Option<std::fs::File>,
}

impl std::fmt::Debug for ExecLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExecLock")
    }
}

impl ExecLock {
    /// Acquire the exclusive execution lock for a course, waiting up to
    /// `wait_secs` for a concurrent holder to finish.
    pub fn acquire(course_dir: &Path, wait_secs: u64) -> Result<Self, CarpenterError> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(course_dir.join(".exec.lock"))
            .map_err(crate::core::store::io_to_store)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(wait_secs);
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(Self { file: Some(file) }),
                Err(_) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(200))
                }
                Err(_) => {
                    return Err(CarpenterError::StoreError(
                        "another notebook execution is in progress for this course \
                         — wait for it to finish and retry"
                            .into(),
                    ))
                }
            }
        }
    }

    /// Release the lock early (drop also releases).
    pub fn release(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }
}

/// How long [`run_nbconvert`] waits for a concurrent execution of the same
/// course before giving up (the holder may legitimately be a slow notebook).
const EXEC_LOCK_WAIT_SECS: u64 = 120;

/// Execute a lesson notebook via nbconvert in the course venv, serialized per
/// course (adr/017).
///
/// Concurrent `lesson execute`/`quiz run` processes race jupyter_client's
/// bind-probe-release kernel port reservation and die with `ZMQError: Address
/// already in use` after a 60 s startup timeout, so kernel launches are
/// serialized with [`ExecLock`] before shelling out.
pub fn run_nbconvert(
    course_dir: &Path,
    lesson_dir: &Path,
    timeout_secs: u64,
) -> Result<Output, CarpenterError> {
    require_uv(uv_available())?;
    let mut lock = ExecLock::acquire(course_dir, EXEC_LOCK_WAIT_SECS)?;
    // Run nbconvert from the lesson dir so the kernel cwd resolves `import
    // helper` (helper.py lives next to the notebook). `uv run` walks up to
    // find the venv.
    let timeout_arg = format!("--ExecutePreprocessor.timeout={timeout_secs}");
    let args = [
        "run",
        "jupyter",
        "nbconvert",
        "--execute",
        "--to",
        "notebook",
        "--inplace",
        "--ExecutePreprocessor.allow_errors=True",
        timeout_arg.as_str(),
        "lesson.ipynb",
    ];
    let result = run_uv_or_store(&args, lesson_dir);
    lock.release();
    result
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

    fn lock_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("carpenter-lock-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn exec_lock_contention_times_out_and_recovers() {
        let dir = lock_dir("timeout");
        let held = ExecLock::acquire(&dir, 1).expect("first acquire");
        let err = ExecLock::acquire(&dir, 1).unwrap_err();
        assert!(
            matches!(err, CarpenterError::StoreError(ref m) if m.contains("in progress")),
            "{err}"
        );
        drop(held);
        ExecLock::acquire(&dir, 1).expect("reacquire after release");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn exec_lock_release_allows_immediate_reacquire() {
        let dir = lock_dir("release");
        let mut lock = ExecLock::acquire(&dir, 1).expect("acquire");
        lock.release();
        ExecLock::acquire(&dir, 1).expect("reacquire after explicit release");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
