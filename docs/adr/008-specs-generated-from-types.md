# ADR-008: specs generated from types

Date: 2026-08-08 · Status: Accepted

## Context
adr/003 made the `howto` generated but left `docs/specs/` (the JSON I/O contracts)
**human-maintained** — the one remaining doc surface that can drift from code.
With adr/007 now forcing every command to carry `///` docs, an example block, and a
paired test, the `*Spec`/`Data` serde types and their representative examples are a
reliable single source to generate the specs from.

## Decision
`xtask gen-specs` generates every envelope/spec **table** in-place inside
`docs/specs/*.md` (target state; see "Current status" below):
- **spec inputs** (02–07) from the `*Spec` serde structs — serialize → example JSON
  + a field table.
- **output contracts** (08–19, 21, 22, 23) from the `Data` enum variants — serialize
  → the `data` column; the input/arg column comes from clap introspection (the same
  `///` surface adr/007 enforces).
- Each type carries a representative example in a **co-located `mod examples`**
  (single source, next to the struct — like a `Default`/doctest), exposed to the
  xtask via one `pub` registry fn (so `missing_docs` hits that one item, not each
  example).

### Region-with-markers
gen-specs replaces **only** the table between `<!-- BEGIN GENERATED -->` and
`<!-- END GENERATED -->` markers. Hand-written narrative outside the markers is
preserved — specs keep their context prose (e.g. `09-lesson`'s full-tree
description) alongside a machine-faithful table. Pure-prose files with no generated
region: `01-envelope`, `20-helper-contract`.

`howto.gen.md` stays whole-file generated. The marker regions join it on the
**never hand-edit** list; `cargo xtask build` regenerates them; CI fails on drift
(`git diff --exit-code`).

This **supersedes** the adr/003 consequence that "detailed JSON I/O contracts live in
`docs/specs/` (human-maintained)."

### Current status (honest)
The generator does not exist yet — there is no `xtask`, no `Data` enum, no `*Spec`
structs. **Today every spec table is hand-maintained** and only `19-howto.md`
actually carries the `BEGIN/END GENERATED` markers. The marker regions + the
generator land together in build-order phase 2; until then this ADR describes the
target, not the present. CI drift enforcement on spec markers is enabled when the
generator ships.

## Consequences
+ Specs can no longer drift from code — the typed `*Spec`/`Data` shapes **are** the
  contract.
+ The mandated `<command>_*` tests (adr/007) are the goldens that validate the
  generated envelopes end-to-end.
+ One atom (the type + its `mod examples`) feeds both enforcement (compile) and the
  generated docs — fully DRY.
− Each `*Spec`/`Data` type must carry an example value in its `mod examples` — small
  authoring cost, co-located with the type.
− Generated markdown tables are less narrative than hand prose. Accepted:
  machine-faithful > pretty.
