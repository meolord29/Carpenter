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

## Contributing to carpenter

For contributors. Just want to use carpenter? Start at the
[README](../README.md). AI agents working in this repo read
[`AGENTS.md`](../AGENTS.md) — it is the authoritative contributor guide; this
section is the human on-ramp.

### Quickstart

```sh
git clone https://github.com/meolord29/Carpenter carpenter
cd carpenter
cargo xtask build        # gen-howto + gen-specs + strict build
cargo test --workspace   # --workspace is required: bare cargo test skips xtask
```

Full command matrix (clippy, fmt, doc, nextest):
[AGENTS.md § Build & test](../AGENTS.md#build--test).

### The dev loop

Two build stages ([design/19](design/19-dev-build.md)):

- **release** — `cargo xtask build --release`: the strict ship build (every
  command self-documents and is tested).
- **dev** — `cargo xtask build --dev`: relaxed gates, so a new command compiles
  before its worked-example and test exist. `--capture-example` writes the
  example atom from a real run.

The end-to-end authoring recipe lives in
[AGENTS.md § Dev authoring loop](../AGENTS.md#dev-authoring-loop---dev).

### Where things live

- [AGENTS.md § How it works](../AGENTS.md#how-it-works-structure) — module map
  and request flow.
- [adr/](adr/) — why the non-obvious decisions were made.

### Releases (nightly + stable main)

Two channels, two long-lived branches
([adr/021](adr/021-nightly-main-channels.md)):

```
ivan/<topic> ──PR──▶ nightly   integration trunk; ground-rules checklist + owner-only
                               approval; each push rolls the `nightly` prerelease
nightly ──PR──▶ main           promotion; publishes immutable stable vX.Y.Z (Latest);
                               the only path into main (ci.yml `guard` enforces it)
```

- **`nightly` (unstable)** — the rolling prerelease `release.yml` re-creates on
  every push to the `nightly` branch. Canary users soak each build before
  promotion. Install / stay on it:

  ```sh
  curl -LsSf https://github.com/meolord29/Carpenter/releases/download/nightly/install.sh | sh
  carpenter upgrade --channel nightly
  ```

- **stable (`vX.Y.Z`)** — immutable, versioned from `Cargo.toml`, published when
  `nightly` merges into `main`. GitHub marks it **Latest**, so the README
  one-liner and `carpenter upgrade` (default `--channel stable`) follow it.
  Users never see nightly — the README is end-user-only (install + getting
  started; stable).

**PR ground rules** (feature branches → `nightly`;
[`.github/PULL_REQUEST_TEMPLATE.md`](../.github/PULL_REQUEST_TEMPLATE.md),
verified by the owner at review):

1. All unit tests pass.
2. New features ship with unit tests.
3. If the CLI surface or study workflow changed, the QA agent's
   checklist/prompt (`.opencode/agents/carpenter-dev-validate.md`) is updated
   in the same PR.
4. The PR explains what the feature does.
5. A carpenter-dev-validate report is attached when the PR changes
   learner-facing or course-building surface — the learning simulation ran
   smoothly over existing features and any new ones. Infra/docs/release-process
   PRs record an `N/A — no course-surface change` with the targeted contract
   validation that ran instead.

**Promotion checklist** (nightly → main PR):
1. Bump `version` in `Cargo.toml` (and `Cargo.lock`) in the promotion PR.
2. Merge; CI tags `v<version>` and publishes stable; the smoke lanes verify the
   published artifact via the `/latest/` one-liner.

**Rollback**: stable tags are immutable — install any previous `vX.Y.Z` by
substituting its tag for `latest` in the install URL.

**Branch protection** (`nightly` + `main`): require PRs, require review from
codeowners (`CODEOWNERS` is `* @meolord29` — owner-only approval), require
status checks (ci gates + smoke lanes; `main` additionally requires the
`guard` job, which fails any PR into `main` whose head is not `nightly`).

### Contributing flow

Short-lived `ivan/<topic>` branches off `nightly`, green CI, owner-approved
merge (ground rules above), delete after merge. See
[AGENTS.md § Integration & release](../AGENTS.md#integration--release-adr021).
