# Testing

## Compile-time gates (block the binary — adr/007)
- `#![deny(missing_docs)]` — every public item needs a `///` (clap reads it as
  `--help`).
- `build.rs` syn scan — each command fn (`pub fn -> Result<Data, CarpenterError>`
  in `commands/`) must have (a) a fenced example block in its `///` and (b) a
  `#[test] fn <command>_*` in the same module (name-prefix mapping). Miss →
  `exit(1)`, no binary. Convention: `commands/` holds only command fns (helpers
  live in `core/`).

## Runtime tests (`cargo test`)
- Autouse temp dirs. The parametrized envelope smoke test over every command **is**
  the set of mandated `<command>_*` tests — they double as the goldens that validate
  the generated spec contracts (adr/008).
- Howto stale-check: regenerates `howto.gen.md` to a buffer, asserts it equals the
  committed file — so `cargo test` catches drift locally, not only in CI.
- Skill determinism: re-renders `SKILL.md` via `core/skill.rs`, asserts byte-equal +
  frontmatter validates (adr/009).
- Rust compare + Python helper compare parity tests. Integration: create → fill
  stub → `quiz run` → assert `pass_or_fail`/`last_check` written; load generated
  `helper.py` via Python to score a real fn. Sync preservation test (3-way via
  `scaffold_hash`).

Gates: `rustfmt`, `clippy -D warnings`, `cargo doc --no-deps -- -D warnings`.
