# ADR-007: compile-enforced command self-documentation

Date: 2026-08-08 · Status: Accepted

## Context
carpenter's "automate everything verifiable" and "code is self-documenting"
principles are only as strong as their enforcement. The howto is generated
(adr/003), but nothing forces a command to carry help text, an example, or a test:
a command could ship with a bare signature, the howto would scrape empty help, the
generated specs would lack an example, and the command would be untested. We want
it to be **impossible to compile a command that is not self-documenting** — the
binary must fail to build.

Two of the four requirements are enforceable by `rustc` (doc strings, function
docs); two are not (examples, unit tests) — there is no rustc/clippy lint for
"this command has an example" or "this command has a test."

## Decision
Two compile-blocking gates:

1. **`#![deny(missing_docs)]`** at the crate root. Every public item (command
   structs, command fns, models, core fns, enum variants, struct fields) must carry
   a `///` doc comment. `clap` derive reads the `///` on a command struct as `about`
   and on an arg field as `help` — so the doc comment **is** the `--help` text.
   Covers the `--help` doc-string requirement and the function-doc requirement for
   the public surface.

2. **`build.rs` syn scanner** (`[build-dependencies] syn`). Runs before compile;
   `cargo:rerun-if-changed=src/commands`. A "command" is every `pub fn` in
   `src/commands/` returning `Result<Data, CarpenterError>`. For each it asserts:
   - the concatenated `#[doc]` attrs contain ≥1 fenced code block (the **example**);
   - a `#[test] fn <command>_*` exists in the same module (**name-prefix** mapping,
     e.g. `create` → `create_ok`, `create_rejects_dup`).
   On a miss: `eprintln!("commands/{file}:{cmd}: missing example | missing test")`
   + `exit(1)` → no binary.

### Alternatives rejected
- **Custom proc-macro `#[command(example=…)]`** — true compile failure and best DX,
  but a new macro crate + `syn`/`quote` deps. build.rs gives the same hard failure
  at lower setup cost and runs on every `cargo build`.
- **xtask check + `#[test]`** — does not fail `cargo build`, only `cargo test` /
  `xtask build`; the requirement is "fail compiling into a binary."
- **Doctest-as-test** — would make the example *be* the test; rejected because
  examples and unit tests are distinct requirements.
- **Inert `#[command_test]` attribute** — reintroduces a marker; name-prefix needs
  none and matches the `Result<Data, _>` signature convention already in
  `03-architecture.md`.

## Consequences
+ Impossible to compile an undocumented or untested command — the binary won't build.
+ The mandated `///` + example block is the **single atom** scraped into `howto`
  (adr/003) and rendered into the generated specs (adr/008). Enforcement and
  generation read one source.
+ The mandated `<command>_*` tests double as the envelope goldens that validate the
  generated spec contracts (adr/008).
+ `missing_docs` forces terse `///` on every public model field/variant → verbose
  models. Accepted (machine-faithful surface).
− `syn` build-dep adds a little cold-build time; mitigated by `rerun-if-changed`.
− Private fns remain undocumented (crate-wide `missing_docs` covers `pub` only; the
  `clippy::missing_docs_in_private_items` layer is deliberately not adopted).
− A helper `pub fn -> Result<Data, _>` in `commands/` would be mis-scanned as a
  command → convention: `commands/` holds **only** command fns; helpers live in `core/`.

## Update (2026-08-10): example atom moved to a file

The per-command **example** is no longer a fenced block in the fn's `///`. It is now
a hand-authored worked-example file at `docs/examples/<module>/<fn>.md` (invocation
+ full `--spec` input + result envelope + notes). Drivers:

- **Discoverability.** The howto previously showed only a one-line opaque envelope
  per command (`"content":"{goals, links}"`), so the spec *shapes* (e.g. plan's
  `goal_index_<i>`) lived only in `docs/specs/` and had to be reverse-engineered.
  The file carries the full spec inline.
- **Single source (DRY).** `xtask gen-howto` embeds the file verbatim into
  `src/howto.gen.md`; nothing is scraped from `///`. The `///` keeps its summary line
  for `--help`/rustdoc (`missing_docs` still applies); the fenced block was retired
  to avoid two example sources drifting.

The gate is unchanged in spirit: `build.rs` still fails the build, but now asserts
**example-file presence** (`docs/examples/<module>/<fn>.md`) instead of a fenced
block in `///`. The `#[test] fn <name>_*` requirement is untouched. `xtask/src/howto.rs`
reads the directory (keyed `<module>::<name>`) and dropped its `syn` dependency;
`cargo xtask gen-howto` regenerates the manual, and `howto_gen_md_is_fresh` catches
drift. One file per CLI leaf (58 at this writing), 1:1 with the gated command fns.
