# ADR-012: Cross-platform path resolution (Linux/macOS/Windows)

Date: 2026-08-11 · Status: Accepted

## Context
carpenter ran on Linux. Two facts forced an explicit cross-platform decision:

1. **`config_dir` was already portable** via the `dirs` crate (`dirs::config_dir()` →
   `~/.config` Linux, `~/Library/Application Support` macOS, `%APPDATA%` Windows), but
   the rest of the path surface was Unix-shaped:
   - `Config::default` `bin_dir` fallback was the literal `/usr/local/bin`
     ([adr/004](004-build-install-split.md) assumed a Unix `~/.local/bin`).
   - `install`/`upgrade` joined the bare name `"carpenter"`, dropping the `.exe`
     Windows requires (`CreateProcess` looks for `carpenter.exe`).
   - `dirs::executable_dir()` returns `None` on macOS/Windows, so a pure-`dirs`
     `bin_dir` default cannot work cross-platform — `#[cfg(target_os)]` branches are
     required regardless.
2. **CI did not exist.** The quality gates (`fmt`, `clippy -D warnings`, `cargo test
   --workspace`, `cargo doc`) ran only locally, and two byte-equality stale-check
   tests (`howto_gen_md_is_fresh`, `specs_marker_regions_are_fresh`) would silently
   break under Windows git (`core.autocrlf` flips LF→CRLF on checkout).

The question: where does "which OS am I on" live, and how do paths stay correct without
scattering `#[cfg]` across the codebase?

## Decision
1. **One compile-time platform module: `core/platform.rs`.** OS identification is
   build-time, via `#[cfg(target_os = "...")]`. It owns the only OS-conditional path
   behavior that `dirs` cannot express:
   - `default_bin_dir() -> PathBuf` — Linux/macOS `~/.local/bin`, Windows
     `%LOCALAPPDATA%\Programs\carpenter`.
   - `exe_file_name(base: &str) -> String` — appends `.exe` on Windows, identity
     elsewhere (so `install`/`upgrade` resolve `carpenter.exe`).
2. **`dirs` is retained for `config_dir`** (`store::config_dir`). It already does the
     right thing per-OS; `platform.rs` does not duplicate it. Single chokepoint stays.
3. **`Config::default` calls `platform::default_bin_dir()`** — the `/usr/local/bin`
     literal is deleted; no hardcoded Unix fallback remains.
4. **CI: a single GitHub Actions matrix over `ubuntu/macos/windows` runs the full gate
   suite** (the same commands in `AGENTS.md` → Build & test), with `uv` installed (the
   `uv_is_available_in_this_env` test asserts it) and a committed `rust-toolchain.toml`
   pin for reproducibility.
5. **`.gitattributes` pins LF** (`* text=auto eol=lf`) so the byte-equality stale-checks
   hold on a Windows checkout.

## Consequences
+ One home for per-OS behavior; no `#[cfg]` scattered outside `core/platform.rs`. The
  existing single `cfg!(windows)` PATH-separator branch in `store::is_on_path` stays
  (it is a runtime split, not a path default).
+ The DB is the source of truth regardless of OS — `course.db`, `config.json`, and the
  lesson layout (`<course>/lessons/<NN>-<slug>/`) are relative paths; `pathlib`'s `/`
  in `helper.py` resolves to `\` on Windows for free. `helper.py` has no shebang, no
  `os.symlink`, no `chmod` — already portable.
+ Generated spec **tables** and howto **example envelopes** stay Linux-illustrative
  (e.g. `~/.local/bin/carpenter`); they are generated from types/`docs/examples/`, not
  per-OS, and the per-OS truth lives here + [design/17](../design/17-cross-platform.md).
− Two `python3`-probe tests (`compare` parity, `helper` validity) spawn `python3`,
  which is absent on Windows; they already `return` on spawn failure, so they **no-op,
  not fail**. Acceptable; optional later: probe `python`/`py` too.
− `upgrade`'s `rename` over a *running* `carpenter.exe` hits a Windows sharing
  violation. Not test-blocking; deferred (write-and-replace-via-temp, or
  rename-on-reboot). Tracked in [design/17](../design/17-cross-platform.md).
− The `String::ends_with("a/b")` assertions (8 sites) had to be normalized to
  `Path`-component checks — `String` suffix-match breaks on `\`. Done as part of the
  cross-platform phase.

Work sequence: [Phase 12 — cross-platform](../design/14-build-order.md). Design detail:
[design/17](../design/17-cross-platform.md).
