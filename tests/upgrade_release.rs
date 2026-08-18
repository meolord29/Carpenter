//! e2e: `upgrade` release mode against a `file://` fixture (adr/018) — the
//! fixture reproduces the release layout (`tarball` + `SHA256SUMS`) using the
//! real test binary, then `upgrade` must download-verify-extract-probe-replace
//! and re-register the skill.

mod common;

use std::path::Path;

use carpenter::commands::upgrade::upgrade;
use carpenter::core::store::Paths;
use carpenter::models::Data;

use common::{release_fixture, unique};

#[test]
fn upgrade_release_mode_installs_and_registers() {
    let Some(target) = common::platform_target() else {
        return; // no published asset on this platform (mapping covered in unit tests)
    };
    let dir = release_fixture(target);
    std::env::set_var(
        "CARPENTER_DOWNLOAD_BASE",
        format!("file://{}", dir.display()),
    );
    let root = unique("root");
    let paths = Paths {
        config_dir: Some(root.join("xdg").join("carpenter")),
        root,
    };
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
