# Cross-platform (Linux / macOS / Windows)

carpenter targets all three desktop OSes. The DB-relative layout (`course.db`,
`config.json`, `<course>/lessons/<NN>-<slug>/`) is portable by construction; the
OS-specific surface is **where the app lives on disk** and **what the binary is named**.
Decision rationale: [adr/012](../adr/012-cross-platform.md). Work sequence:
[Phase 12](14-build-order.md).

## Per-OS path resolution

| concern | Linux | macOS | Windows | owner |
|---|---|---|---|---|
| config dir | `~/.config/carpenter` | `~/Library/Application Support/carpenter` | `%APPDATA%\carpenter` | `store::config_dir` via `dirs` |
| `bin_dir` default | `~/.local/bin` | `~/.local/bin` | `%LOCALAPPDATA%\Programs\carpenter` | `core/platform.rs` (`#[cfg(target_os)]`) |
| installed binary | `carpenter` | `carpenter` | `carpenter.exe` | `core/platform.rs::exe_file_name` |
| `$PATH` separator | `:` | `:` | `;` | `store::is_on_path` (`cfg!(windows)`) |

`dirs` already resolves `config_dir` correctly per-OS; `core/platform.rs` owns the two
behaviors `dirs` cannot (a `bin_dir` default — `dirs::executable_dir()` is `None` on
macOS/Windows — and the executable extension). `xdg_root()` (the opencode-integration
anchor) is `config_dir.parent()`, so it tracks the OS automatically: `~/.config` Linux,
`%APPDATA%` Windows.

## The platform module (`core/platform.rs`)
Compile-time OS identification — the **only** place `#[cfg(target_os)]` lives outside the
one `cfg!(windows)` PATH-split in `store::is_on_path`:
- `default_bin_dir() -> PathBuf` — the `Config::default` `bin_dir` (replaces the deleted
  `/usr/local/bin` Unix literal).
- `exe_file_name(base: &str) -> String` — `.exe` on Windows, identity elsewhere; used by
  `install`/`upgrade` so `target/release/carpenter.exe` resolves.

## Portability fixes (the Phase 12 inventory)
Audited against `cargo test --workspace` on Windows; grouped by severity:

**Must-fix (test failures):**
- `.gitattributes` → `* text=auto eol=lf`. The stale-checks
  (`howto_gen_md_is_fresh`, `specs_marker_regions_are_fresh`) assert byte-equality
  against committed files; Windows `autocrlf` flips LF→CRLF and breaks them.
- 8 `String::ends_with("a/b")` assertions → `Path`-component checks. `String`
  suffix-match is byte-wise and breaks on `\`. Sites: `install`, `register`,
  `deregister`, `skill` (×2), `bug`, `feature`, `bugfile`.

**Should-fix (runtime correctness):**
- `install` dest + `upgrade` built/dest → `platform::exe_file_name` (drop the bare
  `"carpenter"` join so `carpenter.exe` is produced).

**Clean (zero work — already portable):** symlinks (none used), Unix-only std APIs
(none), shell/shebang invocation (none — subprocesses spawn real binaries), exec-bit /
`chmod` logic (none), `HELPER_PY` (no shebang; `pathlib` only; DB via `parents[2]`),
env-var casing (`std::env` is case-insensitive on Windows).

## Graceful skips on Windows (acceptable, no fix)
Two tests spawn `python3` (`core/compare.rs` parity, `core/helper.rs` validity). Windows
ships `python`/`py`, not `python3`; both probes `return` on spawn failure, so they
**no-op, not fail**. Optional later: probe `python`/`py` as a fallback.

## Known limitation
`upgrade` renames a new binary over the *running* `carpenter.exe`; Windows locks the
executing image and the rename hits a sharing violation. Not exercised by tests; deferred
(write-and-replace via a non-running temp name, or `MoveFileEx` rename-on-reboot).

## CI
`.github/workflows/ci.yml` — a single job, matrix `os: [ubuntu-latest, macos-latest,
windows-latest]`, runs the **full** gate suite (the `AGENTS.md` → Build & test commands):
checkout → `rust-toolchain.toml` (stable, pinned) → `Swatinem/rust-cache` →
`astral-sh/setup-uv` (for `uv_is_available_in_this_env`) → `cargo fmt --check` →
`cargo clippy --workspace --all-targets -- -D warnings` → `cargo xtask build` →
`cargo test --workspace` → `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`.
The generated `howto`/spec surfaces stay Linux-illustrative; CI does not regenerate them
per-OS (they are type-driven, not path-driven).
