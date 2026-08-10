//! App config (`~/.config/carpenter/config.json`).
//!
//! Keys: `bin_dir`, `python`, `timeout_secs`, `active_course`, `source_dir`.
//! Defaults applied for any missing key; an unknown key in the file is ignored on
//! load (rejected by the `config set` command).

use std::path::{Path, PathBuf};

use crate::core::error::CarpenterError;
use crate::core::store;

/// The app config.
#[derive(Debug, Clone)]
pub struct Config {
    /// Where `install` places the binary (default `~/.local/bin`).
    pub bin_dir: PathBuf,
    /// Python version for `venv create` (`None` = uv default).
    pub python: Option<String>,
    /// Per-cell execution timeout in seconds (default 30).
    pub timeout_secs: u64,
    /// Active course slug (set by `course switch`).
    pub active_course: Option<String>,
    /// Carpender source checkout (used by `upgrade` to resolve `--source`).
    pub source_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bin_dir: dirs::home_dir()
                .map(|h| h.join(".local/bin"))
                .unwrap_or_else(|| PathBuf::from("/usr/local/bin")),
            python: None,
            timeout_secs: 30,
            active_course: None,
            source_dir: None,
        }
    }
}

/// Load config from a specific file; defaults for any missing/unreadable file.
pub fn load_from(path: &Path) -> Config {
    let mut cfg = Config::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        return cfg;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return cfg;
    };
    if let Some(s) = v.get("bin_dir").and_then(|x| x.as_str()) {
        cfg.bin_dir = PathBuf::from(s);
    }
    cfg.python = v.get("python").and_then(|x| x.as_str()).map(String::from);
    if let Some(n) = v.get("timeout_secs").and_then(|x| x.as_u64()) {
        cfg.timeout_secs = n;
    }
    cfg.active_course = v
        .get("active_course")
        .and_then(|x| x.as_str())
        .map(String::from);
    cfg.source_dir = v
        .get("source_dir")
        .and_then(|x| x.as_str())
        .map(PathBuf::from);
    cfg
}

/// Save config to a specific file (atomic).
pub fn save_to(path: &Path, cfg: &Config) -> Result<(), CarpenterError> {
    let v = serde_json::json!({
        "bin_dir": cfg.bin_dir,
        "python": cfg.python,
        "timeout_secs": cfg.timeout_secs,
        "active_course": cfg.active_course,
        "source_dir": cfg.source_dir,
    });
    store::atomic_write(path, v.to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn tmp() -> PathBuf {
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        std::env::temp_dir().join(format!("carpenter-cfg-{}-{n}.json", std::process::id()))
    }

    #[test]
    fn load_missing_file_gives_defaults() {
        let cfg = load_from(Path::new("/nonexistent/carpenter-config.json"));
        assert_eq!(cfg.timeout_secs, 30);
        assert!(cfg.active_course.is_none());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let path = tmp();
        let cfg = Config {
            active_course: Some("ds".into()),
            timeout_secs: 99,
            ..Config::default()
        };
        save_to(&path, &cfg).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded.active_course.as_deref(), Some("ds"));
        assert_eq!(loaded.timeout_secs, 99);
        let _ = std::fs::remove_file(&path);
    }
}
