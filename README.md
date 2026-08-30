# carpenter

A tool your AI coding agent uses to build and grade Python practice courses.

<p align="center"><img src="assets/logo.png" alt="carpenter logo" width="360"></p>

Tell the agent what you want to learn, like "teach me the 20% of statistics
that gets 80% of the value from my marketing data."
Approve its outline, then fill in the notebook exercises and hit Run for an
instant grade.

Carpenter tracks your weak spots, and the agent drills them with fresh
exercises until they clear.

> `v0.8.0` · Linux/macOS · works with
> [opencode](https://opencode.ai) and claude code · Apache-2.0

## Where it fits

- **Online course platforms** decide the curriculum and pad it with material
  you didn't ask for. Here, the course contains only what your goal needs.
- **Public notebook repos** (GitHub/Kaggle-style) are reading material —
  unstructured, and nobody grades your code. This gives ordered lessons with
  graded exercises.
- **Paper-explainer AI tutors** explain papers. This teaches the other half:
  writing code for your own data problems.

No app, no web UI, no bundled course library — it needs an AI coding agent.

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
