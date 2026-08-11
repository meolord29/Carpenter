//! Output examples for `build`/`install`/`upgrade` (docs/specs/18-build-install-upgrade.md).

use serde_json::json;

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::json;
    use crate::models::Data;

    /// `(cmd, input, note, data)` rows for the build/install/upgrade output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "build <path>",
                "target dir",
                "scaffolds course.json + course.db + lessons/",
                Data::Build {
                    path: String::from("/courses/ds"),
                    slug: String::from("ds"),
                    created: vec![
                        String::from("course.json"),
                        String::from("course.db"),
                        String::from("lessons/"),
                    ],
                },
            ),
            (
                "install [--bin-dir <p>]",
                "—",
                "`on_path` = whether `bin_dir` resolves on `$PATH`",
                Data::Install {
                    installed: true,
                    bin: String::from("~/.local/bin/carpenter"),
                    on_path: true,
                },
            ),
            (
                "upgrade [--source <p>] [--bin-dir <p>] [--no-skill]",
                "source dir (`--source` → config `source_dir` → error)",
                "`skill` outcomes: `{refreshed:true,…}` · `{refreshed:false,reason:\"not_registered\",warning:\"…\"}` · `--no-skill` ⇒ `skill:null`",
                Data::Upgrade {
                    upgraded: true,
                    version: env!("CARGO_PKG_VERSION").into(),
                    bin: String::from("~/.local/bin/carpenter"),
                    source: String::from("/src/carpenter"),
                    skill: Some(json!({"refreshed": true, "app": "opencode", "path": "~/.config/opencode/skills/carpenter/SKILL.md"})),
                },
            ),
        ]
    }
}
