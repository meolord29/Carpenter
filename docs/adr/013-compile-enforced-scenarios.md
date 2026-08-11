# ADR-013: compile-enforced multi-command scenarios

Date: 2026-08-11 · Status: Accepted

## Context

Per-command worked examples ([adr/007](007-compile-enforced-command-docs.md)) show
each CLI leaf in isolation. They do not show *compositions* — the real
end-to-end flows the agent is expected to drive (`course create` → `plan create` →
`plan confirm` → `lesson create` → `quiz run` → `progress summary`). Without a
second category of example, the agent learns commands but not workflows.

We want **scenario** files (one per workflow) that compose several public command
fns toward a goal. But a scenario with no enforcement can silently degrade into a
one-liner (a single command repeated, or two commands with padding) and still
claim to be a "workflow." We want it to be **impossible to compile a scenario
that is not a real multi-fn flow** — the same property [adr/007](007-compile-enforced-command-docs.md)
buys for per-command self-documentation.

## Decision

Add a second `build.rs` gate over `examples/*.md` (the new scenario directory at
the repo root, distinct from `docs/examples/`). For each scenario file:

1. Parse fenced ```sh blocks and find lines invoking `carpenter`.
2. Skip leading global flags (`-c <course>`, `--course <course>`, `--root <p>`),
   then read `<group> <fn>`.
3. Resolve each to the **known command set** — the same signature-based
   identification [adr/007](007-compile-enforced-command-docs.md) already uses
   (`pub fn -> Result<Data, CarpenterError>` in `src/commands/`, read as
   `<group>::<fn>`). Count **distinct** hits (same fn twice = 1).
4. Assert the file references **≥3** distinct command fns.

Plus a **global floor**: at least one scenario file must exist (raisable later).
A miss → `eprintln!` + `exit(1)` → no binary.

### "Tag" on public functions

There is no new attribute or decorator. The "tag" that makes a function countable
*is* its existing signature — `pub fn -> Result<Data, CarpenterError>` in
`commands/`. The scenario parser resolves `carpenter <group> <fn>` invocations
against this set; nothing is annotated on the Rust side. This reuses the single
command-identification mechanism the build already maintains.

### Alternatives rejected

- **Proc-macro `#[command]` attribute** — explicit and visible, but a new
  proc-macro crate + `syn`/`quote` deps. Conflicts with the lean-deps principle;
  [adr/007](007-compile-enforced-command-docs.md) rejected the same idea for the
  per-command gate (build.rs gives the same hard failure at lower cost).
- **xtask check + `#[test]`** — fails `cargo test`, not `cargo build`. The
  requirement is "fail compiling into a binary," which only build.rs satisfies
  (same reasoning as [adr/007](007-compile-enforced-command-docs.md)).
- **Per-fn coverage mandate** (every command fn must appear in ≥1 scenario) —
  maximal, but forces trivial commands (`howto`, `config get`) into narratives
  they don't fit, and raises authoring cost sharply. Rejected in favor of an
  optional-but-floored model: scenarios grow organically; only the ≥3-fn rule per
  file and a ≥1-file global floor are enforced.
- **Floor of 4** — guarantees richer flows but invites padding for narrower
  scenarios. ≥3 is the floor; it is a single `const`, trivially raisable.
- **Title-only howto entries** — rejected; scenarios are embedded **verbatim**
  into the howto (rich agent context, matches the [adr/007](007-compile-enforced-command-docs.md)
  update's verbatim embedding of per-command examples).

## Consequences

+ Impossible to ship a fake or padded scenario — the binary won't build.
+ The agent gains composition knowledge the per-command atoms can't convey.
+ **Free skill inclusion**: `core/skill.rs::render()` already inlines all of
  `manual::MANUAL` into `SKILL.md` ([adr/009](009-skill-assembled-from-fields.md)
  update), so a `## Scenarios` section in the howto flows into the skill
  automatically. One authored pointer line directs the agent to it.
+ Reuses the existing command-identification mechanism — no new "tag," no new
  macro, no second source of the command set.
− `cargo:rerun-if-changed=examples` and a markdown/invocation parser add a
  little build.rs weight; mitigated by `rerun-if-changed`.
− `.md`-only convention under `examples/` (Cargo auto-discovers `examples/*.rs`
  as example targets — harmless for markdown, but a stray `.rs` would be picked
  up). Documented in [design/18](../design/18-scenarios.md); set `autoexamples =
  false` if paranoia is ever warranted.
− Parsing shell invocations from markdown is a lighter-weight contract than the
  syn-based command scan; the parser is deliberately narrow (leading global flags
  + `<group> <fn>`) and reports unknown invocations by name so they can be fixed
  or registered.
