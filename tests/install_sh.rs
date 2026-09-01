//! e2e: `scripts/install.sh` as a subprocess against a `file://` release
//! fixture — the installer UX contract (adr/024): branded banner, install
//! plan, channel-correct tagline (the stable release's tag-patched copy must
//! drop the `nightly` mark), non-interactive auto-proceed, and the interactive
//! consent gate (declining installs nothing). Sandbox: per-test `HOME` +
//! `XDG_CONFIG_HOME`, so nothing touches real user config.

mod common;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use common::{config_root, platform_target, release_fixture, unique};

fn installer_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("install.sh")
}

fn setup(tag: &str) -> PathBuf {
    let root = unique(tag);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn unix_chmod(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

/// Fake `opencode` on PATH so app detection is deterministic.
fn stub_opencode(root: &Path) {
    let stub = root.join("stub");
    std::fs::create_dir_all(&stub).unwrap();
    std::fs::write(stub.join("opencode"), "#!/bin/sh\nexit 0\n").unwrap();
    unix_chmod(&stub.join("opencode"));
}

fn run_installer(script: &Path, root: &Path, fixture: &Path) -> std::process::Output {
    Command::new("sh")
        .arg(script)
        .env("HOME", root)
        .env("XDG_CONFIG_HOME", config_root(root))
        .env(
            "CARPENTER_DOWNLOAD_BASE",
            format!("file://{}", fixture.display()),
        )
        .env("CARPENTER_INSTALL_DIR", root.join("bin"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("stub").display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .unwrap()
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The non-interactive lane (no TTY): banner + full install plan print, the
/// consent prompt is skipped, the run proceeds and installs.
#[test]
fn install_sh_prints_plan_and_proceeds_non_interactive() {
    let Some(target) = platform_target() else {
        return;
    };
    if !Command::new("sh").arg("-c").arg("exit 0").status().is_ok() {
        return;
    }
    let fixture = release_fixture(target);
    let root = setup("install-plan");
    stub_opencode(&root);
    let bin = root.join("bin").join("carpenter");

    let out = run_installer(&installer_script(), &root, &fixture);
    assert!(
        out.status.success(),
        "install.sh failed: {}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(text.contains("carpenter installer  ·  nightly"), "{text}");
    assert!(text.contains("install plan"), "{text}");
    assert!(
        text.contains(&format!("carpenter-{target}.tar.gz")),
        "{text}"
    );
    assert!(text.contains("register  opencode ->"), "{text}");
    assert!(
        text.contains("non-interactive — proceeding with the plan above"),
        "{text}"
    );
    assert!(text.contains("✓ installed carpenter"), "{text}");
    assert!(
        !text.contains("proceed with the install plan?"),
        "non-interactive run must not print the consent prompt: {text}"
    );
    assert!(bin.is_file(), "binary installed at {}", bin.display());

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&fixture);
}

/// The tagline is channel-correct: the repo script (TAG="nightly") shows the
/// canary mark; the stable release's tag-patched copy must show the plain
/// `carpenter installer` and no `nightly` anywhere. This also pins the
/// `TAG="nightly"` line shape that `.github/workflows/release.yml` seds — if
/// the anchor moves, this fails before a release ships a broken installer.
#[test]
fn install_sh_banner_is_channel_correct() {
    let Some(target) = platform_target() else {
        return;
    };
    if !Command::new("sh").arg("-c").arg("exit 0").status().is_ok() {
        return;
    }
    let script = installer_script();
    let text = std::fs::read_to_string(&script).unwrap();
    let anchor = "TAG=\"nightly\"";
    assert_eq!(
        text.matches(anchor).count(),
        1,
        "release.yml seds this exact line; keep one TAG anchor"
    );
    let patched = text.replacen(anchor, "TAG=\"v9.9.9\"", 1);
    let stable_script = root_stub("install-channel");
    std::fs::write(&stable_script, patched).unwrap();

    let fixture = release_fixture(target);
    let root = setup("install-channel-home");
    stub_opencode(&root);
    let out = run_installer(&stable_script, &root, &fixture);
    assert!(
        out.status.success(),
        "tag-patched install.sh failed: {}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(text.contains("carpenter installer"), "{text}");
    assert!(
        !text.contains("nightly"),
        "stable installer output must not mention nightly: {text}"
    );

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&fixture);
    let _ = std::fs::remove_dir_all(stable_script.parent().unwrap());
}

/// The interactive consent gate: under a real pty (`script(1)`), answering
/// `n` aborts before anything is downloaded or installed. Skips where no pty
/// helper exists; macOS `script` does not propagate the child exit status, so
/// assertions ride on output + filesystem state instead.
#[test]
fn install_sh_declined_consent_installs_nothing() {
    if platform_target().is_none() {
        return;
    }
    let has_script = Command::new("sh")
        .arg("-c")
        .arg("command -v script >/dev/null 2>&1")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !has_script {
        return; // no pty helper (unusual container); CI covers Linux + macOS
    }
    let root = setup("install-decline");
    stub_opencode(&root);
    let script = installer_script();
    let bin_dir = root.join("bin");

    let mut cmd = Command::new("script");
    if cfg!(target_os = "macos") {
        cmd.args(["-q", "/dev/null", "sh"]).arg(&script);
    } else {
        cmd.args(["-qec", &format!("sh {}", script.display()), "/dev/null"]);
    }
    let mut child = cmd
        .env("HOME", &root)
        .env("XDG_CONFIG_HOME", config_root(&root))
        .env("CARPENTER_DOWNLOAD_BASE", "file:///nonexistent")
        .env("CARPENTER_INSTALL_DIR", &bin_dir)
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("stub").display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"n\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
    .replace('\r', "");

    assert!(
        text.contains("proceed with the install plan?"),
        "consent prompt must appear under a pty: {text}"
    );
    assert!(
        text.contains("aborted"),
        "declining must abort with a clear message: {text}"
    );
    assert!(
        !bin_dir.join("carpenter").exists(),
        "declined install must not write a binary"
    );
    assert!(
        !config_root(&root).join("opencode").exists(),
        "declined install must not register anything"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A scratch dir for the tag-patched script copy (sibling cleanup in the test).
fn root_stub(tag: &str) -> PathBuf {
    let dir = unique(tag);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("install.sh")
}
