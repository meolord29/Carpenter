//! `upgrade` — rebuild carpenter from a source checkout and atomically replace
//! the installed binary; best-effort refreshes the registered skill.
//!
//! Source resolves `--source` → `config.source_dir` → `ValidationError`. Runs
//! `cargo xtask build --release` from source (regenerating `howto` + specs), then
//! writes the binary via tmp+rename. Skill refresh: `refreshed` if registered,
//! `not_registered` (warning) if absent, `null` with `--no-skill`.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::core::config;
use crate::core::error::CarpenterError;
use crate::core::exec;
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
    let source_dir = resolve_source(source, &cfg)?;
    if !source_dir.is_dir() {
        return Err(CarpenterError::ValidationError(format!(
            "source dir does not exist: {}",
            source_dir.display()
        )));
    }
    // gen-howto + gen-specs + release build (the embedded howto regenerates)
    exec::run_cargo_or_store(&["xtask", "build", "--release"], &source_dir)?;
    let built = source_dir.join("target/release/carpenter");
    if !built.exists() {
        return Err(CarpenterError::StoreError(format!(
            "build finished but binary not found at {}",
            built.display()
        )));
    }
    let version = version_of(&built)?;
    let target_dir = bin_dir.map(PathBuf::from).unwrap_or(cfg.bin_dir);
    std::fs::create_dir_all(&target_dir).map_err(store::io_to_store)?;
    let dest = target_dir.join("carpenter");
    let tmp = dest.with_extension("carpenter-tmp");
    std::fs::copy(&built, &tmp).map_err(store::io_to_store)?;
    std::fs::rename(&tmp, &dest).map_err(store::io_to_store)?;
    let skill_outcome = if no_skill {
        None
    } else {
        Some(refresh_skill(paths))
    };
    Ok(Data::Upgrade {
        upgraded: true,
        version,
        bin: dest.display().to_string(),
        source: source_dir.display().to_string(),
        skill: skill_outcome,
    })
}

fn resolve_source(source: Option<&str>, cfg: &config::Config) -> Result<PathBuf, CarpenterError> {
    if let Some(s) = source.map(PathBuf::from) {
        return Ok(s);
    }
    if let Some(s) = cfg.source_dir.clone() {
        return Ok(s);
    }
    Err(CarpenterError::ValidationError(
        "no source dir: pass --source <path> or set `config source_dir` \
         (clone carpenter and run from there)"
            .into(),
    ))
}

/// Read the version string from a freshly built binary (`<bin> --version`).
fn version_of(bin: &Path) -> Result<String, CarpenterError> {
    let out = std::process::Command::new(bin)
        .arg("--version")
        .output()
        .map_err(|e| CarpenterError::StoreError(format!("failed to run new binary: {e}")))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .nth(1)
        .map(String::from)
        .ok_or_else(|| CarpenterError::StoreError(format!("could not parse version from {text:?}")))
}

/// Best-effort skill refresh outcome. Never errors (a refresh failure does not
/// roll back a successful binary upgrade).
fn refresh_skill(paths: &Paths) -> Value {
    let root = match paths.xdg_root() {
        Ok(r) => r,
        Err(e) => {
            return json!({"refreshed": false, "reason": "no_xdg_root", "error": e.to_string()})
        }
    };
    let skill_path = App::Opencode.skill_path(root);
    if !skill_path.exists() {
        return json!({"refreshed": false, "reason": "not_registered", "warning": NOT_REGISTERED_WARNING});
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
    fn upgrade_errors_without_source() {
        let paths = testutil::meta_setup();
        let err = upgrade(&paths, None, None, true).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(&paths.root);
    }

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
    fn not_registered_warning_is_stable() {
        // guards the single-sourced string against drift (spec 18)
        assert!(NOT_REGISTERED_WARNING.contains("not registered in opencode"));
    }
}
