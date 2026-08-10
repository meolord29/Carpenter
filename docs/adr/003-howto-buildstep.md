# ADR-003: howto is generated at build time by scraping the clap surface

Date: 2026-08-08 · Status: Accepted

## Context
The agent must be able to discover carpenter's full command surface and contracts.
The reference tool (leeteacher) used a hand-maintained 290-line `howto.py` that
drifted from the real CLI. We need self-documentation that cannot drift.

## Decision
An **`xtask`** binary in the workspace constructs the real `clap::Command` tree,
introspects it (subcommands, args, help text), and emits `src/howto.gen.md`. The
binary embeds it via `include_str!("howto.gen.md")`, and `carpenter howto` prints it.
`cargo xtask gen-howto` runs whenever commands/flags change; `howto.gen.md` is never
hand-edited and CI fails if it is stale.

## Consequences
+ Zero hand-maintained prose; the howto is always a faithful reflection of the CLI.
+ Single scrape target (the compiled `clap` tree) — no second source of truth.
+ The same introspection can seed a richer agent manual or the `link register`
  manifest later.
− Two-step build: `gen-howto` before `build` (wrapped as `cargo xtask build`).
− ~~Detailed JSON I/O contracts live in `docs/specs/` (human-maintained), not in the
  scraped howto; the howto covers the command surface, specs.md covers the envelopes.~~
  *(Superseded by [adr/008](008-specs-generated-from-types.md): spec **tables** are
  now generated from types between markers. Narrative outside markers remains
  hand-maintained; the howto still covers only the command surface.)*
