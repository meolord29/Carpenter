# Scenarios

A **scenario** is a hand-authored markdown file that walks a multi-command
workflow end-to-end — composing several public command fns to reach a final
result. It is the *composition* companion to the single-command worked example
(`docs/examples/<module>/<fn>.md`, [adr/007](../adr/007-compile-enforced-command-docs.md)):
the per-command atom shows one leaf; a scenario shows a real flow.

## Why

Per-command examples teach leaves, not compositions. An agent that has only ever
seen `lesson create` in isolation does not know it follows `plan confirm` and
feeds `quiz run`. Scenarios capture the *workflows* the agent is expected to
drive, and — because they are gated and generated, like everything else — they
cannot drift or degrade into a one-liner.

## Location & format

Scenario files live at `examples/<name>.md` (repo root) — distinct from
`docs/examples/` (the per-command atoms). One file = one end-to-end flow.

Format (mirrors the per-command example style, multi-step):

- An `# H1` title (the scenario's display name in the howto).
- A short narrative of the goal and the course it targets.
- A sequence of fenced ```sh blocks, each a `carpenter …` invocation, optionally
  followed by its result envelope / commentary.
- The final result envelope.

A scenario is the **single source** for the flow it describes — it is embedded
verbatim into the howto by `xtask gen-howto` and never hand-duplicated (DRY).

## Compile-time gate ([adr/013](../adr/013-compile-enforced-scenarios.md))

`build.rs` (which already enforces per-command self-documentation) gains a second
gate over `examples/`:

1. **Known command set** — the same signature-based identification used today:
   every `pub fn -> Result<Data, CarpenterError>` in `src/commands/`, read as
   `<group>::<fn>`. No new attribute or decorator — the "tag" on a public
   function *is* this signature (reused, per [adr/013](../adr/013-compile-enforced-scenarios.md)).
2. **Per-file parse** — for each `examples/*.md`, walk fenced ```sh blocks, find
   lines invoking `carpenter`, skip leading global flags (`-c <course>`,
   `--course <course>`, `--root <p>`), and read `<group> <fn>`. Resolve each
   against the known set; count **distinct** hits (the same fn twice = 1).
3. **Floors** — assert each scenario references **≥3** distinct command fns, and
   that **≥1 scenario file exists** (the global floor, raisable later). A miss →
   `eprintln!` + `exit(1)` → no binary.

The ≥3 floor is a single `const`, so it is trivially raisable to 4 (or higher)
without touching the parser. Unknown invocations (typos, not-yet-landed commands)
are reported by name so the author can fix or register them.

## Generation flow

```
examples/*.md  ──build.rs──>  ≥3-fn gate (blocks the binary)
                └─xtask gen-howto─>  ## Scenarios section in howto.gen.md
                                        └─core/skill.rs::render()─>  inlined into SKILL.md
```

- **`xtask gen-howto`** reads `examples/*.md` **sorted by filename** (for
  determinism) and appends a `## Scenarios` section after the per-command
  groups. Each scenario's full body is embedded **verbatim** (rich agent context;
  matches how per-command examples are embedded per the [adr/007](../adr/007-compile-enforced-command-docs.md)
  update).
- **`core/skill.rs`** — `render()` already inlines all of `manual::MANUAL`
  (= `howto.gen.md`) into `SKILL.md` under `## Command manual`
  ([adr/009](../adr/009-skill-assembled-from-fields.md) update). Scenarios
  therefore flow into the skill **for free**. One authored pointer line is added
  to the skill body so the agent is directed to the inlined scenarios rather
  than having to discover them.

Scenarios do **not** appear in `docs/specs/` — specs are per-command I/O contract
tables ([adr/008](../adr/008-specs-generated-from-types.md)); scenarios are
howto material, not envelope contracts.

## Determinism & drift

- Sorted-by-filename read → the generated `## Scenarios` section is reproducible.
- `howto_gen_md_is_fresh` (the existing stale-check) regenerates the whole
  `howto.gen.md` to a buffer and asserts byte-equality — it covers the new
  section at no extra cost. `cargo xtask gen-howto` refreshes it after a change.
- `render_is_deterministic` covers the skill, as today.

## Cargo note

`examples/` is also Cargo's auto-discovered example-target directory (per-crate
`[[example]]` auto-discovery picks up `examples/*.rs`). Scenario files are
`.md`-only, so Cargo ignores them and the gate/build are unaffected. If a stray
`.rs` ever lands there, either move it or set `autoexamples = false` in the
`[package]` manifest. Convention: **`.md` only** under `examples/`.

## What goes in a scenario

| Good scenario | Bad / rejected |
|---|---|
| A real end-to-end flow (`course → plan → confirm → lesson → quiz → progress`) | A single command repeated (fails the distinct-fn floor) |
| ≥3 distinct command fns composed toward one goal | Two commands with padding to clear the floor |
| One file per workflow; narrative ties the steps together | Duplicating a per-command worked example verbatim |
| `.md` under `examples/` | Anything executable / non-markdown |
