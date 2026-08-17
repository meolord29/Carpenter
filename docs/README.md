# carpenter docs

Five areas. The **code is the source of truth**; these are the human-readable
mirrors plus the generated contracts. When code and docs disagree, the code wins
— update the docs.

| area | what lives here | start with |
|------|-----------------|------------|
| [design/](design/) | architecture + rationale, one concern per file | [01-overview](design/01-overview.md) |
| [data-model/](data-model/) | SQLite schema mirror (ER, DDL, conventions, status) | [data-model/README](data-model/README.md) |
| [specs/](specs/) | per-command I/O contracts (tables generated from types, adr/008) | [01-envelope](specs/01-envelope.md) |
| [adr/](adr/) | architecture decision records (append-only history) | [adr/README](adr/README.md) |
| [examples/](examples/) | one worked example per CLI leaf — the howto's single source (adr/007) | any `<module>/<fn>.md` |

## Read order (newcomer)

1. [design/01-overview](design/01-overview.md) — what carpenter is.
2. [design/03-architecture](design/03-architecture.md) — module map + stack.
3. [specs/01-envelope](specs/01-envelope.md) — the JSON envelope every command emits.
4. [data-model/README](data-model/README.md) — the schema.
5. [adr/README](adr/README.md) — why the decisions were made.

## Generated — never hand-edit

- `src/howto.gen.md` (whole file) — `cargo xtask gen-howto`.
- The `<!-- BEGIN/END GENERATED -->` table regions in `specs/*.md` —
  `cargo xtask gen-specs`. Drift is caught inside `cargo test`.

## Do not consolidate (build-coupled)

`examples/` (one file per command fn, keyed by `build.rs` — adr/007) and the
generated regions in `specs/` (keyed by filename in `xtask gen-specs` — adr/008)
are coupled to the build; merging them breaks it. `adr/` is append-only history.
The free-prose area is `design/` (and it stays one concern per file).
