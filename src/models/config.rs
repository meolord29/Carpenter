//! Output examples for `config` commands (docs/specs/16-config.md).

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use crate::models::Data;

    /// `(cmd, input, note, data)` rows for the `config` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "get",
                "—",
                "all keys with defaults applied; optionals `null` when unset",
                Data::ConfigAll {
                    bin_dir: String::from("~/.local/bin"),
                    python: None,
                    timeout_secs: 30,
                    active_course: None,
                    source_dir: None,
                },
            ),
            (
                "get <key>",
                "key",
                "unknown key ⇒ `ValidationError`",
                Data::ConfigGet {
                    key: String::from("timeout_secs"),
                    value: serde_json::json!(30),
                },
            ),
            (
                "set <key> <value>",
                "key + value",
                "value coerced to the key's type (`timeout_secs`⇒int); unknown key ⇒ `ValidationError`",
                Data::ConfigSet {
                    key: String::from("timeout_secs"),
                    value: serde_json::json!(45),
                },
            ),
        ]
    }
}
