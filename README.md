# carpenter

A CLI that an AI coding agent drives to build and grade Python practice courses.

<p align="center"><img src="assets/logo.png" alt="carpenter logo" width="360"></p>

carpenter is a Rust CLI designed to be operated by an AI coding agent (such as
[opencode](https://opencode.ai) or claude code), not by you. You tell the agent
what you want to learn; the agent writes the lessons, practice problems, and
quizzes. carpenter stores everything in SQLite, renders ordinary Jupyter
notebooks from it, and grades your code the moment you run it — deterministically,
so a pass is a real pass rather than the agent being agreeable.

The learning environment stays close to everyday Python work: you edit a normal
notebook, your code runs in a normal [`uv`](https://docs.astral.sh/uv/)-managed
venv, and the practice format is plain fill-in-a-function stubs. There is no
proprietary sandbox or custom environment to learn around.

carpenter also records live pass/fail state for every practice problem and quiz.
The agent can read that state, see exactly where you keep failing, and generate
additional practice for those weak spots until they clear.

> Experimental (`v0.8.0`) · Linux/macOS · Apache-2.0

## How it works

1. Open your agent app and describe what you want to learn — ideally a problem
   you actually have, e.g. "clean up my lab's CSV exports with pandas".
2. The agent drafts a course outline (lessons, goals). Nothing is built until
   you approve it.
3. Each lesson renders to a Jupyter notebook: teaching sections, then practice
   stubs and a quiz. You fill in the functions and hit **Run**.
4. carpenter executes your code against test cases and reports **PASS / FAIL**
   immediately. Failing items stay visible to the agent, which keeps drilling
   them with new practice.

## Where it fits

Compared to the usual alternatives:

- **Online course platforms** offer fixed curricula. They decide what you learn
  and often pad it with material your goal doesn't require. A carpenter course
  contains only what your stated goal needs — nothing else gets built.
- **Public notebook repos** (GitHub/Kaggle-style) are reading material:
  unstructured, and nobody tells you whether your own code works. carpenter
  gives you ordered lessons and grades the code you write.
- **Paper-explainer AI tutors** help you understand academic papers. carpenter
  teaches the other half: writing code that solves your own data-related
  problems — analysis, scripting, automation.

What carpenter is not: there is no app or web UI, no bundled course library,
and it requires an AI coding agent to drive it. It is experimental software.

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
