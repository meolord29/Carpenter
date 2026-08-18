//! `uninstall` — remove the carpenter skill from opencode, delete the
//! installed binary, and optionally purge the config (adr/019).
//!
//! Ordering: skill first (recoverable via `register`), binary last (point of
//! no return). Removing the running binary is safe on Linux/macOS — unlink
//! keeps the inode alive for the running process (the same property
//! `upgrade`'s copy-replace relies on). Course data is never touched.

use std::path::PathBuf;

use serde_json::{json, Value};

use crate::core::config;
use crate::core::error::CarpenterError;
use crate::core::skill::{self, App};
use crate::core::store::{self, Paths};
use crate::models::Data;

/// `uninstall [--bin-dir <p>] [--purge-config]`.
pub fn uninstall(
    paths: &Paths,
    bin_dir: Option<&str>,
    purge_config: bool,
) -> Result<Data, CarpenterError> {
    let cfg = paths
        .config_file()
        .map(|p| config::load_from(&p))
        .unwrap_or_default();
    let target_dir = bin_dir.map(PathBuf::from).unwrap_or(cfg.bin_dir);
    let bin = target_dir.join("carpenter");
    let skill_path = App::Opencode.skill_path(paths.xdg_root()?);
    let config_path = paths.config_file();
    if !skill_path.exists() && !bin.exists() {
        return Err(CarpenterError::NotFound(format!(
            "carpenter is not installed (no skill at {} and no binary at {})",
            skill_path.display(),
            bin.display()
        )));
    }
    let skill_outcome = remove_skill(paths);
    let removed_bin = if bin.exists() {
        std::fs::remove_file(&bin).map_err(store::io_to_store)?;
        Some(bin.display().to_string())
    } else {
        None
    };
    let config_purged = if purge_config {
        config_path
            .map(|p| p.exists() && std::fs::remove_file(&p).map_err(store::io_to_store).is_ok())
            .unwrap_or(false)
    } else {
        false
    };
    Ok(Data::Uninstall {
        uninstalled: true,
        bin: removed_bin,
        skill: skill_outcome,
        config_purged,
    })
}

/// Remove the opencode skill, best-effort: a missing or failing removal never
/// fails the uninstall (mirrors `upgrade`'s `skill_outcome_for`).
fn remove_skill(paths: &Paths) -> Value {
    let root = match paths.xdg_root() {
        Ok(r) => r,
        Err(e) => return json!({"removed": false, "reason": "no_xdg_root", "error": e.to_string()}),
    };
    match skill::deregister(App::Opencode, root) {
        Ok(d) => json!({"removed": true, "app": d.app, "path": d.path}),
        Err(e) => json!({"removed": false, "reason": "not_registered", "error": e.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    #[test]
    fn uninstall_removes_skill_and_binary() {
        let paths = testutil::meta_setup();
        crate::commands::register::register(&paths, "opencode", false).expect("register");
        let bin_dir = paths.root.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let bin = bin_dir.join("carpenter");
        std::fs::write(&bin, b"fake binary").unwrap();
        let Data::Uninstall {
            uninstalled,
            bin: removed_bin,
            skill,
            config_purged,
        } = uninstall(&paths, Some(bin_dir.to_str().unwrap()), false).expect("uninstall")
        else {
            panic!("Uninstall");
        };
        assert!(uninstalled);
        assert_eq!(removed_bin.as_deref(), Some(bin.to_str().unwrap()));
        assert!(!bin.exists(), "binary should be gone");
        assert!(!skill_path_of(&paths).exists(), "skill should be gone");
        assert_eq!(skill["removed"], json!(true));
        assert!(!config_purged);
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn uninstall_binary_only_reports_not_registered_skill() {
        let paths = testutil::meta_setup();
        let bin_dir = paths.root.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("carpenter"), b"fake binary").unwrap();
        let Data::Uninstall { skill, bin, .. } =
            uninstall(&paths, Some(bin_dir.to_str().unwrap()), false).expect("uninstall")
        else {
            panic!("Uninstall");
        };
        assert_eq!(skill["removed"], json!(false));
        assert_eq!(skill["reason"], json!("not_registered"));
        assert!(bin.is_some());
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn uninstall_skill_only_reports_null_bin() {
        let paths = testutil::meta_setup();
        crate::commands::register::register(&paths, "opencode", false).expect("register");
        let bin_dir = paths.root.join("empty-bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let Data::Uninstall { bin, skill, .. } =
            uninstall(&paths, Some(bin_dir.to_str().unwrap()), false).expect("uninstall")
        else {
            panic!("Uninstall");
        };
        assert!(bin.is_none(), "no binary was present");
        assert_eq!(skill["removed"], json!(true));
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn uninstall_not_found_when_nothing_installed() {
        let paths = testutil::meta_setup();
        let bin_dir = paths.root.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let err = uninstall(&paths, Some(bin_dir.to_str().unwrap()), false).unwrap_err();
        assert!(matches!(err, CarpenterError::NotFound(_)), "{err:?}");
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn uninstall_bin_path_is_directory_is_store_error() {
        let paths = testutil::meta_setup();
        crate::commands::register::register(&paths, "opencode", false).expect("register");
        let bin_dir = paths.root.join("bin");
        std::fs::create_dir_all(bin_dir.join("carpenter")).unwrap();
        let err = uninstall(&paths, Some(bin_dir.to_str().unwrap()), false).unwrap_err();
        assert!(matches!(err, CarpenterError::StoreError(_)), "{err:?}");
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn uninstall_purges_config_only_with_flag() {
        let paths = testutil::meta_setup();
        let bin_dir = paths.root.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("carpenter"), b"fake binary").unwrap();
        let cfg = paths.config_file().expect("config file");
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&cfg, b"{}").unwrap();
        let Data::Uninstall { config_purged, .. } =
            uninstall(&paths, Some(bin_dir.to_str().unwrap()), false).expect("keep config")
        else {
            panic!("Uninstall");
        };
        assert!(!config_purged);
        assert!(cfg.exists(), "config must survive without --purge-config");
        std::fs::write(bin_dir.join("carpenter"), b"fake binary").unwrap();
        let Data::Uninstall { config_purged, .. } =
            uninstall(&paths, Some(bin_dir.to_str().unwrap()), true).expect("purge config")
        else {
            panic!("Uninstall");
        };
        assert!(config_purged);
        assert!(!cfg.exists(), "config should be purged");
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    fn skill_path_of(paths: &Paths) -> std::path::PathBuf {
        App::Opencode.skill_path(paths.xdg_root().unwrap())
    }
}
