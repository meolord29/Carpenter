//! Output shapes for `venv` commands.

use serde::Serialize;

/// One installed package (from `uv pip list`).
#[derive(Debug, Clone, Serialize)]
pub struct Package {
    /// package name.
    pub name: String,
    /// installed version.
    pub version: String,
}

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use super::Package;
    use crate::models::Data;

    /// `(cmd, input, note, data)` rows for the `venv` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "create [--python 3.12]",
                "python version",
                "`StoreError` if no uv; `AlreadyExists` if `.venv` present",
                Data::VenvCreate {
                    course: String::from("ds"),
                    python: String::from("3.12"),
                    path: String::from("<root>/courses/ds/.venv"),
                    deps: vec![
                        String::from("jupyterlab"),
                        String::from("nbconvert"),
                        String::from("nbclient"),
                        String::from("ipykernel"),
                    ],
                },
            ),
            (
                "sync",
                "—",
                "",
                Data::VenvSync {
                    course: String::from("ds"),
                    synced: true,
                },
            ),
            (
                "list",
                "—",
                "",
                Data::VenvList {
                    course: String::from("ds"),
                    packages: vec![Package {
                        name: String::from("nbconvert"),
                        version: String::from("7.16.4"),
                    }],
                },
            ),
            (
                "add <pkg>",
                "package name (repeatable)",
                "",
                Data::VenvAdd {
                    course: String::from("ds"),
                    added: vec![String::from("numpy")],
                    packages: vec![Package {
                        name: String::from("numpy"),
                        version: String::from("2.1.3"),
                    }],
                },
            ),
        ]
    }
}
