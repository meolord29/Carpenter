# carpenter

**Your AI agent builds the course. You practice. carpenter grades — instantly.**

Stop hunting for the perfect tutorial. Tell your agent what you want to learn,
approve the outline it drafts, and start practicing on a notebook that grades you
the second you hit Run.

<p align="center"><img src="assets/logo.png" alt="carpenter logo" width="360"></p>

## Why it clicks

- **Built for you, not pre-baked.** Say what you want to learn → your agent drafts
  the lessons, the practice problems, and the quizzes.
- **Grading you can trust.** The agent teaches, but carpenter scores — so a pass is
  a real pass, not the AI being nice to you.
- **You stay in control.** Nothing gets built until you approve the outline.
- **Live progress.** Ask the agent how you're doing — it shows what's done and where
  you're stuck.

> Experimental (`v0.7.1`) · Python/Jupyter · works with
> [opencode](https://opencode.ai) and claude code · Apache-2.0

## Try it

1. [Install](#install) carpenter (one-time).
2. Open opencode. Tell the agent what you want to learn.
3. Approve the outline, fill in a practice stub, hit **Run** → **PASS / FAIL**.

That's the whole loop. You never touch the CLI — the agent does.

## Install

One line:

```sh
curl -LsSf https://github.com/meolord29/Carpenter/releases/latest/download/install.sh | sh
```

The binary lands in `~/.local/bin` (add it to `PATH` if the installer says so).
Update later with `carpenter upgrade` (it also refreshes the skill).

If `opencode` or `claude` is on your machine, the installer detects it and asks
whether to register the carpenter skill (each app, one by one); you can also
register manually:

```sh
carpenter register --app opencode      # or: claude-code
```

## Learn more

- [DEV.md](DEV.md) — building and contributing to carpenter.
- `carpenter howto` — the full command manual.

## License

Apache 2.0. See [`LICENSE`](LICENSE).
