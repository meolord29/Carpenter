# carpenter

**An AI agent builds your Python course. carpenter grades it — instantly, deterministically.**

Tell your agent what you want to learn. It drafts an outline, you approve it, and
carpenter renders an interactive Jupyter notebook: teaching cells, practice stubs,
and quizzes that score themselves the moment you hit Run. Stop researching the
"best" resource — start practicing on one built for you.

The split that makes it work: **the LLM is the creative tutor; carpenter is the
deterministic backbone.** Storage, rendering, and grading live in a Rust binary
over SQLite — reproducible, inspectable, no "the AI said I'm right" false greens.
The agent decides what to teach; carpenter keeps score.

- **Generate, don't search.** Describe what you want to learn → your agent drafts a
  course outline → you approve → a rendered notebook with teaching, practice, and
  quizzes. Kills tutorial hell.
- **Instant, trustworthy grading.** A verification-only `helper.py` scores every
  check live and writes `pass_or_fail` to SQLite. It never prints the expected
  answer; compare rules are parity-tested Rust ↔ Python.
- **Tight feedback loop.** Practice attaches to its teaching section; quizzes cap
  each lesson; `progress` / `notes` / `skip` let you steer. Status derives
  bottom-up — no manual bookkeeping.
- **Deterministic backbone.** One Rust binary, a single SQLite file, `uv`-managed
  course venv, docs generated from the code (a drift test fails the build if they
  go stale). Creative tutor; reproducible scoreboard.

> **Status:** experimental / early. `v0.1.0` — all core build phases have landed;
> no stability guarantees yet. Python/Jupyter only; opencode today (`claude-code` /
> `agents` stubbed behind `--app`). Apache-2.0.

<!-- TODO: 30-sec terminal GIF here — tell agent → approve outline → fill stub → INSTANT PASS → progress summary -->

## Agent-driven, not manual

You don't drive carpenter — your AI agent does. Tell it what you want to learn;
it does the rest.

But you stay in control. Before a single lesson gets built, the agent walks you
through the plan — what each lesson covers, the practice problems, the quizzes —
and waits for your OK. Nothing gets generated behind your back.

And it's trustworthy by design: **the agent teaches, but carpenter grades.** The
agent never touches the scoring, so a pass is a real pass — not the AI being nice
to you.

Works with [opencode](https://opencode.ai) today; see [Install](#install). You
can run it by hand too (`carpenter howto` prints the full manual) — it's just
built for an agent to drive.

## How it works

1. `course create --spec -` → `course.json` + empty `course.db`.
2. `plan create` (course) → draft; user approves → `plan confirm` materializes
   goal rows + `covered_by` links.
3. `lesson create --spec -` → DB inserts; render notebook (managed cells) + a
   generic, verification-only `helper.py`.
4. Learner fills a practice stub, runs the check cell → helper scores and writes
   `pass_or_fail` / `last_check` (instant feedback).
5. `quiz run` → `uv run jupyter nbconvert --execute` in the course venv → helper
   cells score every quiz; scaffolding errors escalate via `scaffold_hash`,
   learner errors are scored as fails.
6. `progress summary` rolls up lessons / quizzes / goals / notes; lesson & goal
   status derive bottom-up from `pass_or_fail` + `skip`.

## Install

From a source checkout (the canonical path — `install` copies the *running*
binary, so build first, then run the built binary):

```sh
git clone https://github.com/meolord29/Carpenter carpenter
cd carpenter
cargo xtask build --release        # gen-howto + gen-specs + optimized build

./target/release/carpenter install   # copy the built binary onto PATH
carpenter register --app opencode    # write the opencode skill + permission
```

What each step does:

- `cargo xtask build --release` — the canonical build: regenerates the `howto`
  manual + spec tables, then compiles an optimized binary to
  `target/release/carpenter`.
- `carpenter install` — copies the *currently running* binary into `bin_dir`
  (default `~/.local/bin`; override with `--bin-dir <path>` or set it via
  `carpenter config set bin_dir <path>`). The envelope reports `on_path` so you
  know whether `~/.local/bin` is on your `$PATH` (add it if not).
- `carpenter register --app opencode` — writes
  `~/.config/opencode/skills/carpenter/SKILL.md` (the skill embeds the generated
  `howto` + version + binary path) and merges `permission.skill.carpenter="allow"`
  into `~/.config/opencode/opencode.json` so it loads without prompting.
  `claude-code` / `agents` are accepted by `--app` but not yet implemented.

Verify: `carpenter --version` and `carpenter howto`. To self-update later from a
pulled checkout: `carpenter upgrade` (rebuilds, replaces the binary, and refreshes
the registered skill if present).

## Quickstart

**Prerequisites:** [carpenter installed](#install) and [`uv`](https://github.com/astral-sh/uv)
on `PATH`. Learner execution needs `uv` (venv + `nbconvert`); `helper.py` itself is
Python stdlib-only.

```sh
# create a course from a spec on stdin
carpenter course create --spec - <<'EOF'
{ "title": "Data Structures", "slug": "data-structures",
  "goal": "Understand core data structures from the ground up",
  "description": "Arrays, lists, hashing, trees." }
EOF

carpenter course switch data-structures

# author a lesson (renders lesson.ipynb + helper.py)
#   spec shape: docs/examples/lesson/create.md
carpenter -c data-structures lesson create --spec lesson.json

# set up the course venv, then run every quiz in the lesson notebook
carpenter -c data-structures venv create --python 3.12
carpenter -c data-structures quiz run arrays-101

# live progress
carpenter -c data-structures progress summary
```

Every command returns one envelope on stdout, e.g.:
```json
{"status":"ok","message":"quizzes run: arrays-101","data":{"lesson_id":"arrays-101","quizzes":[{"quiz_id":"q1","passed":1,"total":1}],"saved":true}}
```

Run `carpenter howto` for the full, always-current command manual (one worked
example per command).

## Commands

| group | commands | purpose |
|-------|----------|---------|
| `course` | create list show update delete switch | course CRUD + active-course |
| `plan` | create show list confirm update delete | human-in-the-loop goals (draft → confirm) |
| `goal` | add list update remove | course objectives (status derives or pins) |
| `lesson` | create get list show update delete sync execute | author + render + lifecycle + run notebook |
| `quiz` | run list show results | end-of-lesson assessment (nbconvert) |
| `venv` | create sync list add | uv-managed course venv |
| `skip` | — | exclude a lesson / quiz / practice from derivation |
| `progress` | show summary | live roll-up |
| `notes` | add show list update resolve remove | qualitative tracker (gaps, mistakes, strengths, …) |
| `bug` / `feature` | file list show resolve | file-backed feedback under `~/.config/carpenter/` |
| `config` | get set | app defaults |
| `register` / `deregister` | — | agent-app skill integration |
| `build` / `install` / `upgrade` | — | scaffold + self-install + self-upgrade |
| `link` | register | future CLI registry manifest |
| `howto` | — | print the generated command manual |

Exact input/output (spec JSON shapes + envelope `data`): `carpenter howto` and
[`docs/specs/`](docs/specs/README.md). Global flags: `--version`, `--root <path>`,
`-c` / `--course <slug>` (defaults to `active_course` in config).

## Learn more

- `carpenter howto` — the full, always-current command manual (one worked example per command).
- [`docs/specs/`](docs/specs/README.md) — exact command I/O contracts (envelope shapes).
- [`AGENTS.md`](AGENTS.md) — how carpenter works internally + contributor guide (build, layout, conventions, editing rules).
- [`docs/design/`](docs/design/README.md) · [`docs/data-model/`](docs/data-model/README.md) · [`docs/adr/`](docs/adr/README.md) — full design, schema, and decisions.

## License

Apache License 2.0. See [`LICENSE`](LICENSE).
