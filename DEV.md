# Developing carpenter

For contributors. Just want to use carpenter? Start at the [README](README.md).
AI agents working in this repo read [`AGENTS.md`](AGENTS.md) — it is the
authoritative contributor guide; this file is the human on-ramp.

## Quickstart

```sh
git clone https://github.com/meolord29/Carpenter carpenter
cd carpenter
cargo xtask build        # gen-howto + gen-specs + strict build
cargo test --workspace   # --workspace is required: bare cargo test skips xtask
```

Full command matrix (clippy, fmt, doc, nextest):
[AGENTS.md § Build & test](AGENTS.md#build--test).

## The dev loop

Two build stages ([design/19](docs/design/19-dev-build.md)):

- **release** — `cargo xtask build --release`: the strict ship build (every
  command self-documents and is tested).
- **dev** — `cargo xtask build --dev`: relaxed gates, so a new command compiles
  before its worked-example and test exist. `--capture-example` writes the
  example atom from a real run.

The end-to-end authoring recipe lives in
[AGENTS.md § Dev authoring loop](AGENTS.md#dev-authoring-loop---dev).

## Where things live

- [docs/](docs/) — the five docs areas + newcomer read order; start at
  [docs/README.md](docs/README.md).
- [AGENTS.md § How it works](AGENTS.md#how-it-works-structure) — module map
  and request flow.
- [docs/adr/](docs/adr/) — why the non-obvious decisions were made.

## Releases (branch-governed channels)

Two channels, governed by two long-lived branches
([adr/020](docs/adr/020-branch-governed-channels.md)):

```
ivan/<topic> ──PR──▶ main          trunk; gates + PR smoke; publishes nothing
main ──merge──▶ pre-release        rolls the `edge` prerelease (canary channel)
pre-release ──merge──▶ release     publishes stable vX.Y.Z (marked Latest)
```

- **`edge` (unstable)** — a rolling prerelease that `release.yml` re-creates on
  every push to `pre-release`. Canary users soak each build before promotion.
- **stable (`vX.Y.Z`)** — immutable, versioned from `Cargo.toml`, published on
  every push to `release`. GitHub marks it **Latest**, so the README one-liner
  and `carpenter upgrade` (default `--channel stable`) follow it.

**Promotion checklist** (pre-release → release PR):
1. Bump `version` in `Cargo.toml` (and `Cargo.lock`) in the promotion PR.
2. Merge; CI tags `v<version>` and publishes stable; the smoke lanes verify the
   published artifact via the `/latest/` one-liner.

**Rollback**: stable tags are immutable — install any previous `vX.Y.Z` by
substituting its tag for `latest` in the install URL.

`--channel edge` on `carpenter upgrade` (or the edge install one-liner) opts
into the canary channel.

## Contributing flow

Trunk-based: short-lived `ivan/<topic>` branches off `main`, green CI, delete
after merge. See
[AGENTS.md § Trunk-based development](AGENTS.md#trunk-based-development).
