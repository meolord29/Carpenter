# carpenter

**Your AI agent builds the course. You practice. carpenter grades — instantly.**

Stop hunting for the perfect tutorial. Tell your agent what you want to learn,
approve the outline it drafts, and start practicing on a notebook that grades you
the second you hit Run.

[ GIF: tell agent → approve → fill stub → PASS → progress ]

## Why it clicks

- **Built for you, not pre-baked.** Say what you want to learn → your agent drafts
  the lessons, the practice problems, and the quizzes.
- **Grading you can trust.** The agent teaches, but carpenter scores — so a pass is
  a real pass, not the AI being nice to you.
- **You stay in control.** Nothing gets built until you approve the outline.
- **Live progress.** Ask the agent how you're doing — it shows what's done and where
  you're stuck.

> Experimental (`v0.5.0`) · Python/Jupyter · works with
> [opencode](https://opencode.ai) · Apache-2.0

## Try it

1. [Install](#install) carpenter (one-time).
2. Open opencode. Tell the agent what you want to learn.
3. Approve the outline, fill in a practice stub, hit **Run** → **PASS / FAIL**.

That's the whole loop. You never touch the CLI — the agent does.

## Install

**Linux (x86_64) or macOS (Apple Silicon)** — one line:

```sh
curl -LsSf https://github.com/meolord29/Carpenter/releases/download/edge/install.sh | sh
```

The binary lands in `~/.local/bin` (add it to `PATH` if the installer says so).
Update later with `carpenter upgrade` (fetches the latest build + refreshes the
skill). If `opencode` is on your machine, the installer also registers the
carpenter skill into it; otherwise register manually:

```sh
carpenter register --app opencode
```
To verify integrity, the release also ships `SHA256SUMS` next to each tarball.

**Any other platform** (incl. Intel Macs) — build from source.
Needs [Rust](https://rustup.rs) and [`uv`](https://github.com/astral-sh/uv):

```sh
git clone https://github.com/meolord29/Carpenter carpenter
cd carpenter
cargo xtask build --release
./target/release/carpenter install
carpenter register --app opencode
```

## Learn more

- [`AGENTS.md`](AGENTS.md) — how it works + contributor guide.
- [`docs/`](docs/) — full design, schema, and command contracts.
- `carpenter howto` — the full command manual.

## License

Apache 2.0. See [`LICENSE`](LICENSE).
