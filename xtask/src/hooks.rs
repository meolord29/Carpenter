//! Drift guard for the versioned git hooks (`.githooks/`).

#[cfg(test)]
#[test]
fn pre_commit_hook_is_present_and_executable() {
    let hook = crate::paths::workspace_root().join(".githooks/pre-commit");
    assert!(hook.is_file(), "missing .githooks/pre-commit");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&hook)
            .expect("hook metadata")
            .permissions()
            .mode();
        assert!(mode & 0o111 != 0, ".githooks/pre-commit must be executable");
    }
}
