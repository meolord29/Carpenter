//! Shared helpers for integration tests (`tests/*.rs`): unique temp dirs,
//! a release-layout fixture (tarball + `SHA256SUMS` from the real test
//! binary), and subprocess env sandboxing (Linux + macOS `dirs` resolution).

#![allow(dead_code)] // each test binary consumes a subset

use std::path::{Path, PathBuf};
use std::process::Command;

use carpenter::core::release::{self, checksum_tool_for};

/// A unique temp dir path for `tag` (caller creates it).
pub fn unique(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("carpenter-it-{tag}-{}-{nanos}", std::process::id()))
}

/// Build a release-layout fixture (tarball + correct SHA256SUMS) from the real
/// carpenter test binary; returns its dir.
pub fn release_fixture(target: &str) -> PathBuf {
    let dir = unique("rel");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_carpenter"), dir.join("carpenter")).unwrap();
    let tarball = format!("carpenter-{target}.tar.gz");
    let status = Command::new("tar")
        .args(["-czf"])
        .arg(dir.join(&tarball))
        .arg("-C")
        .arg(&dir)
        .arg("carpenter")
        .status()
        .unwrap();
    assert!(status.success());
    std::fs::remove_file(dir.join("carpenter")).unwrap();
    let (tool, args) = checksum_tool_for(std::env::consts::OS).unwrap();
    let out = Command::new(tool)
        .args(args)
        .arg(dir.join(&tarball))
        .output()
        .unwrap();
    assert!(out.status.success());
    let hash = String::from_utf8_lossy(&out.stdout);
    let hash = hash.split_whitespace().next().unwrap();
    std::fs::write(dir.join("SHA256SUMS"), format!("{hash}  {tarball}\n")).unwrap();
    dir
}

/// The platform target for this runner, if a release asset is published for it.
pub fn platform_target() -> Option<&'static str> {
    release::platform_target()
}

/// The config root a carpenter subprocess will resolve when spawned with
/// `HOME=<root>` (+ `XDG_CONFIG_HOME`): `~/.config` on Linux,
/// `~/Library/Application Support` on macOS (`dirs::config_dir()` ignores
/// `XDG_CONFIG_HOME` there).
pub fn config_root(root: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        root.join("Library").join("Application Support")
    } else {
        root.join(".config")
    }
}

/// A sandboxed subprocess builder: carpenter spawned with `HOME` and
/// `XDG_CONFIG_HOME` pointed inside `root`, so config resolution stays off
/// the real user config on both Linux and macOS.
pub fn sandboxed(exe: &Path, root: &Path) -> Command {
    let mut cmd = Command::new(exe);
    cmd.env("HOME", root)
        .env("XDG_CONFIG_HOME", config_root(root))
        .arg("--root")
        .arg(root);
    cmd
}

/// The sandboxed opencode paths for `root`: (`SKILL.md`, `opencode.json`).
pub fn opencode_paths(root: &Path) -> (PathBuf, PathBuf) {
    let cfg = config_root(root);
    let skill = cfg
        .join("opencode")
        .join("skills")
        .join("carpenter")
        .join("SKILL.md");
    let perms = cfg.join("opencode").join("opencode.json");
    (skill, perms)
}

/// The sandboxed carpenter config file for `root`.
pub fn config_file(root: &Path) -> PathBuf {
    config_root(root).join("carpenter").join("config.json")
}
