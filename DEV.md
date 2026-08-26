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

## Contributing flow

Trunk-based: short-lived `ivan/<topic>` branches off `main`, green CI, delete
after merge. See
[AGENTS.md § Trunk-based development](AGENTS.md#trunk-based-development).
