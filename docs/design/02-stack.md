# Stack

| concern | crate |
|---------|-------|
| CLI | `clap` (derive; `--help` is free and is what `howto` scrapes) |
| models / validation | `serde` + `validator` |
| storage | `rusqlite` (bundled SQLite) |
| notebooks | `serde_json` (build `.ipynb` v4 cells; metadata as `Value`) |
| subprocess (notebook exec) | `std::process::Command` → `uv run jupyter nbconvert` (lesson execute + quiz run) |
| config dirs | `dirs` (XDG `~/.config/carpenter`) |
| errors | `thiserror` |
| howto codegen | `xtask` binary (see [adr/003](../adr/003-howto-buildstep.md)) |
| spec codegen | `xtask gen-specs` from `*Spec`/`Data` serde types (see [adr/008](../adr/008-specs-generated-from-types.md)) |
| compile-time doc/test gate | `build.rs` + `syn` build-dep (see [adr/007](../adr/007-compile-enforced-command-docs.md)) |

Tooling: `cargo`, `rustfmt`, `clippy`, `cargo-nextest` (or `cargo test`),
`cargo doc --no-deps -- -D warnings`.
