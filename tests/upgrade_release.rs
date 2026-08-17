//! e2e: `upgrade` release mode against a `file://` fixture (adr/018) — the
//! fixture reproduces the release layout (`tarball` + `SHA256SUMS`) using the
//! real test binary, then `upgrade` must download-verify-extract-probe-replace
//! and re-register the skill.

use std::path::{Path, PathBuf};
use std::process::Command;

use carpenter::commands::upgrade::upgrade;
use carpenter::core::release::{self, checksum_tool_for};
use carpenter::core::store::Paths;
use carpenter::models::Data;

fn unique(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("carpenter-it-{tag}-{}-{nanos}", std::process::id()))
}

/// Build a release-layout fixture (tarball + correct SHA256SUMS) from the real
/// carpenter test binary; returns its dir.
fn fixture(target: &str) -> PathBuf {
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

fn paths() -> Paths {
    let root = unique("root");
    Paths {
        config_dir: Some(root.join("xdg").join("carpenter")),
        root,
    }
}

#[test]
fn upgrade_release_mode_installs_and_registers() {
    let Some(target) = release::platform_target() else {
        return; // no published asset on this platform (mapping covered in unit tests)
    };
    let dir = fixture(target);
    std::env::set_var(
        "CARPENTER_DOWNLOAD_BASE",
        format!("file://{}", dir.display()),
    );
    let paths = paths();
    let bin_dir = paths.root.join("bin");

    let Data::Upgrade {
        upgraded,
        version,
        bin,
        source,
        skill,
    } = upgrade(&paths, None, Some(bin_dir.to_str().unwrap()), false).expect("upgrade")
    else {
        panic!("Upgrade");
    };
    assert!(upgraded);
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    assert!(bin.ends_with("bin/carpenter"), "{bin}");
    assert!(
        source.ends_with(&format!("carpenter-{target}.tar.gz")),
        "{source}"
    );
    assert!(Path::new(&bin).is_file(), "binary at {bin}");
    let refreshed = skill.expect("skill written");
    assert_eq!(refreshed["refreshed"], true, "{refreshed}");
    let skill_md = paths
        .root
        .join("xdg")
        .join("opencode")
        .join("skills")
        .join("carpenter")
        .join("SKILL.md");
    assert!(skill_md.is_file(), "skill at {}", skill_md.display());

    // --no-skill variant against the same fixture: binary replaced, skill null
    let Data::Upgrade { skill, .. } =
        upgrade(&paths, None, Some(bin_dir.to_str().unwrap()), true).expect("upgrade again")
    else {
        panic!("Upgrade");
    };
    assert!(skill.is_none());

    std::env::remove_var("CARPENTER_DOWNLOAD_BASE");
    let _ = std::fs::remove_dir_all(&paths.root);
    let _ = std::fs::remove_dir_all(&dir);
}
