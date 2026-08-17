//! `upgrade` — replace the installed carpenter binary, then update the skill.
//!
//! Two modes (adr/016): **release** (default — fetch the GitHub `edge` tarball,
//! verify its checksum, extract, probe, atomically replace; always (re-)registers
//! the skill, mirroring `scripts/install.sh`) and **source** (`--source` or
//! config `source_dir` — rebuild via `cargo xtask build --release`, best-effort
//! skill refresh). `--bin-dir`/`--no-skill` apply to both.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::core::config;
use crate::core::error::CarpenterError;
use crate::core::exec;
use crate::core::release::{self, Staged};
use crate::core::skill::{self, App};
use crate::core::store::{self, Paths};
use crate::models::Data;

/// The single-sourced "not registered" warning (spec 18).
const NOT_REGISTERED_WARNING: &str =
    "CLI not registered in opencode — nothing to upgrade. Run `carpenter register`.";

/// `upgrade [--source <p>] [--bin-dir <p>] [--no-skill]`.
pub fn upgrade(
    paths: &Paths,
    source: Option<&str>,
    bin_dir: Option<&str>,
    no_skill: bool,
) -> Result<Data, CarpenterError> {
    let cfg = paths
        .config_file()
        .map(|p| config::load_from(&p))
        .unwrap_or_default();
    // `_stage` keeps the download dir alive until the binary is copied out.
    let (version, origin, built, _stage) = match resolve_mode(source, &cfg)? {
        Mode::Release => {
            let staged = upgrade_from_release()?;
            (
                release::probe_version(&staged.bin)?,
                staged.url.clone(),
                staged.bin.clone(),
                Some(staged),
            )
        }
        Mode::Source(dir) => {
            let built = build_from_source(&dir)?;
            (
                release::probe_version(&built)?,
                dir.display().to_string(),
                built,
                None,
            )
        }
    };
    let target_dir = bin_dir.map(PathBuf::from).unwrap_or(cfg.bin_dir);
    std::fs::create_dir_all(&target_dir).map_err(store::io_to_store)?;
    let dest = target_dir.join("carpenter");
    let tmp = dest.with_extension("carpenter-tmp");
    std::fs::copy(&built, &tmp).map_err(store::io_to_store)?;
    std::fs::rename(&tmp, &dest).map_err(store::io_to_store)?;
    let skill_outcome = if no_skill {
        None
    } else {
        Some(skill_outcome_for(source.is_some(), paths))
    };
    Ok(Data::Upgrade {
        upgraded: true,
        version,
        bin: dest.display().to_string(),
        source: origin,
        skill: skill_outcome,
    })
}

/// Where this upgrade comes from: the published release or a source checkout.
enum Mode {
    /// GitHub `edge` tarball.
    Release,
    /// Local source dir to rebuild.
    Source(PathBuf),
}

fn resolve_mode(source: Option<&str>, cfg: &config::Config) -> Result<Mode, CarpenterError> {
    if let Some(s) = source.map(PathBuf::from) {
        return Ok(Mode::Source(s));
    }
    if let Some(s) = cfg.source_dir.clone() {
        return Ok(Mode::Source(s));
    }
    Ok(Mode::Release)
}

/// Download + verify + extract the release for this platform into a temp stage
/// dir (removed on drop — adr/016).
fn upgrade_from_release() -> Result<Staged, CarpenterError> {
    let target = release::platform_target().ok_or_else(|| {
        CarpenterError::ValidationError(format!(
            "no release asset for {} {} — build from source with `--source <path>` \
             (published: Linux x86_64, macOS Apple Silicon)",
            std::env::consts::OS,
            std::env::consts::ARCH
        ))
    })?;
    let tmp = release::stage_dir()?;
    release::fetch_release(&release::download_base(), target, &tmp)
}

/// Run `cargo xtask build --release` in `dir`; return the built binary's path.
fn build_from_source(source_dir: &Path) -> Result<PathBuf, CarpenterError> {
    if !source_dir.is_dir() {
        return Err(CarpenterError::ValidationError(format!(
            "source dir does not exist: {}",
            source_dir.display()
        )));
    }
    // gen-howto + gen-specs + release build (the embedded howto regenerates)
    exec::run_cargo_or_store(&["xtask", "build", "--release"], source_dir)?;
    let built = source_dir.join("target/release/carpenter");
    if !built.exists() {
        return Err(CarpenterError::StoreError(format!(
            "build finished but binary not found at {}",
            built.display()
        )));
    }
    Ok(built)
}

/// Release mode always (re-)registers (installer parity); source mode refreshes
/// only when already registered. Best-effort: never fails the upgrade.
fn skill_outcome_for(source_mode: bool, paths: &Paths) -> Value {
    let root = match paths.xdg_root() {
        Ok(r) => r,
        Err(e) => {
            return json!({"refreshed": false, "reason": "no_xdg_root", "error": e.to_string()})
        }
    };
    if source_mode {
        let skill_path = App::Opencode.skill_path(root);
        if !skill_path.exists() {
            return json!({"refreshed": false, "reason": "not_registered", "warning": NOT_REGISTERED_WARNING});
        }
    }
    match skill::register(App::Opencode, root) {
        Ok(r) => json!({"refreshed": true, "app": r.app, "path": r.path}),
        Err(e) => json!({"refreshed": false, "reason": "refresh_failed", "error": e.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    #[test]
    fn upgrade_errors_when_source_dir_missing() {
        let paths = testutil::meta_setup();
        let missing = paths.root.join("no-such-src");
        let err = upgrade(&paths, Some(missing.to_str().unwrap()), None, true).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn upgrade_resolves_source_from_config() {
        let paths = testutil::meta_setup();
        // config source_dir points at a non-dir → ValidationError (proves resolution)
        crate::commands::config::set(&paths, "source_dir", "/definitely/not/here").unwrap();
        let err = upgrade(&paths, None, None, true).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn upgrade_release_mode_fails_cleanly_when_unreachable() {
        // unsupported platforms never reach the download (see target_for tests)
        if release::platform_target().is_none() {
            return;
        }
        let paths = testutil::meta_setup();
        // no --source, no config → release mode; point at an empty base dir.
        // (The only unit test that may touch CARPENTER_DOWNLOAD_BASE.)
        let empty = paths.root.join("empty-release");
        std::fs::create_dir_all(&empty).unwrap();
        std::env::set_var(
            "CARPENTER_DOWNLOAD_BASE",
            format!("file://{}", empty.display()),
        );
        let err = upgrade(&paths, None, None, true).unwrap_err();
        std::env::remove_var("CARPENTER_DOWNLOAD_BASE");
        assert!(matches!(err, CarpenterError::StoreError(_)), "{err}");
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn not_registered_warning_is_stable() {
        // guards the single-sourced string against drift (spec 18)
        assert!(NOT_REGISTERED_WARNING.contains("not registered in opencode"));
    }
}
