//! `config` commands — get/set app config keys with typed coercion.
//!
//! `timeout_secs` is coerced to an int; the other keys are strings. An unknown
//! key ⇒ `ValidationError`. Optionals serialize as `null` when unset.

use std::path::PathBuf;

use crate::core::config::{self, Config};
use crate::core::error::CarpenterError;
use crate::core::store::Paths;
use crate::models::Data;

/// The valid config keys.
const KEYS: [&str; 5] = [
    "bin_dir",
    "python",
    "timeout_secs",
    "active_course",
    "source_dir",
];

fn require_key(key: &str) -> Result<(), CarpenterError> {
    if KEYS.contains(&key) {
        Ok(())
    } else {
        Err(CarpenterError::ValidationError(format!(
            "unknown config key {key:?} (valid: {})",
            KEYS.join(", ")
        )))
    }
}

fn value_of(cfg: &Config, key: &str) -> serde_json::Value {
    match key {
        "bin_dir" => serde_json::json!(cfg.bin_dir.display().to_string()),
        "python" => cfg
            .python
            .clone()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
        "timeout_secs" => serde_json::json!(cfg.timeout_secs),
        "active_course" => cfg
            .active_course
            .clone()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
        "source_dir" => cfg
            .source_dir
            .as_ref()
            .map(|p| serde_json::json!(p.display().to_string()))
            .unwrap_or(serde_json::Value::Null),
        _ => unreachable!("require_key guards all calls"),
    }
}

fn apply(cfg: &mut Config, key: &str, value: &str) -> Result<(), CarpenterError> {
    match key {
        "bin_dir" => cfg.bin_dir = PathBuf::from(value),
        "python" => cfg.python = Some(value.into()),
        "timeout_secs" => {
            cfg.timeout_secs = value.parse::<u64>().map_err(|_| {
                CarpenterError::ValidationError(format!(
                    "timeout_secs must be an int (got {value:?})"
                ))
            })?;
        }
        "active_course" => cfg.active_course = Some(value.into()),
        "source_dir" => cfg.source_dir = Some(PathBuf::from(value)),
        _ => unreachable!("require_key guards all calls"),
    }
    Ok(())
}

/// `config get [key]`: all keys, or one key's value.
pub fn get(paths: &Paths, key: Option<&str>) -> Result<Data, CarpenterError> {
    let path = paths
        .config_file()
        .ok_or_else(|| CarpenterError::StoreError(String::from("no config directory resolved")))?;
    let cfg = config::load_from(&path);
    match key {
        None => Ok(Data::ConfigAll {
            bin_dir: cfg.bin_dir.display().to_string(),
            python: cfg.python,
            timeout_secs: cfg.timeout_secs,
            active_course: cfg.active_course,
            source_dir: cfg.source_dir.as_ref().map(|p| p.display().to_string()),
        }),
        Some(k) => {
            require_key(k)?;
            Ok(Data::ConfigGet {
                key: k.into(),
                value: value_of(&cfg, k),
            })
        }
    }
}

/// `config set <key> <value>`: coerce + persist; echoes `{key, value}`.
pub fn set(paths: &Paths, key: &str, value: &str) -> Result<Data, CarpenterError> {
    require_key(key)?;
    let path = paths
        .config_file()
        .ok_or_else(|| CarpenterError::StoreError(String::from("no config directory resolved")))?;
    let mut cfg = config::load_from(&path);
    apply(&mut cfg, key, value)?;
    config::save_to(&path, &cfg)?;
    Ok(Data::ConfigSet {
        key: key.into(),
        value: value_of(&cfg, key),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    #[test]
    fn get_all_applies_defaults_when_unset() {
        let paths = testutil::meta_setup();
        let Data::ConfigAll {
            timeout_secs,
            python,
            source_dir,
            ..
        } = get(&paths, None).expect("get")
        else {
            panic!("ConfigAll");
        };
        assert_eq!(timeout_secs, 30);
        assert!(python.is_none());
        assert!(source_dir.is_none());
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn get_one_returns_value() {
        let paths = testutil::meta_setup();
        set(&paths, "timeout_secs", "42").unwrap();
        let Data::ConfigGet { key, value } = get(&paths, Some("timeout_secs")).expect("get") else {
            panic!("ConfigGet");
        };
        assert_eq!(key, "timeout_secs");
        assert_eq!(value, serde_json::json!(42));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn get_unknown_key_is_validation_error() {
        let paths = testutil::meta_setup();
        let err = get(&paths, Some("nope")).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn set_coerces_timeout_secs_and_roundtrips() {
        let paths = testutil::meta_setup();
        let Data::ConfigSet { value, .. } = set(&paths, "timeout_secs", "99").expect("set") else {
            panic!("ConfigSet");
        };
        assert_eq!(value, serde_json::json!(99));
        // persists across a fresh load
        let path = paths.config_file().unwrap();
        let cfg = config::load_from(&path);
        assert_eq!(cfg.timeout_secs, 99);
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn set_rejects_non_int_timeout() {
        let paths = testutil::meta_setup();
        let err = set(&paths, "timeout_secs", "soon").unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn set_unknown_key_is_validation_error() {
        let paths = testutil::meta_setup();
        let err = set(&paths, "bogus", "x").unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn set_source_dir_roundtrips() {
        let paths = testutil::meta_setup();
        set(&paths, "source_dir", "/src/carpenter").unwrap();
        let Data::ConfigGet { value, .. } = get(&paths, Some("source_dir")).expect("get") else {
            panic!("ConfigGet");
        };
        assert_eq!(value, serde_json::json!("/src/carpenter"));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }
}
