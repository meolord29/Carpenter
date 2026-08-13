---
name: carpenter-dev-validate
description: Use when validating a NEW or changed carpenter command fn. Drives the autonomous dev-build → validate → release loop in an isolated .sandbox (uv required). Front-load triggers: "validate carpenter command", "dev build", "release build", "new command fn", "--capture-example".
---

# carpenter command validation loop

Autonomously validates a new or changed carpenter command fn end-to-end through the
two-stage build, in a self-contained `.sandbox` that is torn down at the end. You
(the agent) simulate real usage and **self-evaluate**; the **human adjudicates**
your pass/fail calls before release.

## Hard prerequisites & boundaries

- **uv MUST be present.** Verify with `carpenter dev check` first; if `uv` is not
  ok, STOP and tell the user to install uv. The agent never installs uv.
- **The agent holds NO filesystem/uv permissions directly.** All sandbox lifecycle
  (create/teardown) and uv interaction go through the CLI:
  `carpenter dev {check,setup,clean}` and `carpenter … venv …`. Never call `rm`,
  `mkdir`, or `uv` yourself.
- **Two build stages only** ([design/19](../../docs/design/19-dev-build.md),
  [adr/016](../../docs/adr/016-dev-feature.md)):
  - dev — `cargo xtask build --dev` (relaxed gates + `--capture-example`).
  - release — `cargo xtask build --release` (strict; the ship build).
- Read `README.md` and `AGENTS.md` to ground how dev/release builds and the
  envelope contract work before starting.

## Inputs to gather from the user (one prompt)

Ask the user for, in a single turn:
1. **Which command fn** is being validated (`<module>::<fn>`, e.g. `lesson::create`).
   If unclear, ask — do not guess.
2. **A topic** for the realistic example content (e.g. "list comprehensions",
   "SVD", "recursion"). This seeds the spec you will build.
3. **Course-from-scratch params**: `title`, `slug`, `goal`, `description` (the
   user provides; you may propose from the topic and ask them to confirm/adjust).

## Phase A — Prereq check
```
carpenter dev check
```
Parse `data.checks[]`. If the `uv` check is `ok:false` → **STOP**, report, and ask
the user to install uv. Do not proceed to setup.

## Phase B — Dev build + sandbox
```
carpenter dev upgrade    # rebuild the dev binary + refresh the local .opencode skill
carpenter dev setup
```
`dev upgrade` is the dev analog of release `upgrade` — it runs
`cargo build --features dev` (producing `./target/debug/carpenter`) and then
refreshes the local skill at `.opencode/skills/carpenter/SKILL.md`. Run it
whenever the CLI surface changes so the agent's in-context manual stays current.
(`carpenter dev register` does the skill refresh alone, without rebuilding.)

`dev setup` creates `./.sandbox` and returns its absolute `path`. **Every
Phase-C carpenter invocation** runs from the repo root with the sandbox isolated:
```
HOME=$PWD/.sandbox ./target/debug/carpenter --root .sandbox <…>
```
This keeps config (`HOME`-derived), courses, and the `.venv` entirely inside
`.sandbox`. After `course create`, run `course switch <slug>` once so later
commands can omit `-c`.

## Phase C — Validate (autonomous)
Run everything inside the sandbox env. Parse every envelope (`status`, `code`,
`data`) and record observed vs expected.

1. **Scaffold**: `course create` (user params) → `venv create` → `venv add <topic
   deps>` (e.g. `numpy` for a numerics topic). `lesson execute` / `quiz run` /
   `lesson verify` REQUIRE the venv — that is why uv is mandatory.
2. **Capture the atom**: run the new command with
   `--capture-example docs/examples/<module>/<fn>.md` against a realistic spec
   built from the topic. The atom's behavioral note is left as a `TODO` — you
   draft it from observations in Phase E.
3. **Simulate real-life usage**: drive the full chain the command belongs to
   (e.g. course → plan create → plan confirm → **[new command]** → lesson execute
   → quiz run → progress summary). Prove the command integrates: its output
   IDs/state resolve in the next command.
4. **Probe error paths** (each must yield the right `code`, never a panic, never a
   bare crash):
   - malformed/missing-required spec field → `ValidationError`
   - duplicate create → `AlreadyExists`
   - destructive op without `--force` → `Conflict`
   - missing prerequisite (e.g. execute before venv) → `StoreError`
5. **Self-evaluate** against the rubric below → build the pass/fail table.

### Validation rubric
| # | check | pass criterion |
|---|---|---|
| 1 | Happy path | `status:ok`; `data` matches the `Data` variant (`src/models/<group>.rs`); side effects verified by a follow-up `show`/`list`. |
| 2 | Error paths | each bad input → correct `code` + `details`; exit 1; no panic. |
| 3 | Chaining | output IDs/state resolve in the next command; the minimal real workflow completes. |
| 4 | Atom fidelity | the `--capture-example` envelope byte-matches the real run; YAML block + invocation are correct. |
| 5 | Venv-gated steps | execute/quiz/verify actually run in the sandbox venv (uv present). |

## Phase D — Human adjudication gate (STOP)
Present a table and **wait**. Do not edit, build, or clean until the user signs off.

```
| check | expected | observed | your verdict | evidence |
```

For each row the human makes the call — you do NOT auto-finalize:
- **PASS** → human says "confirmed correct" or "actually suspect, re-check".
- **FAIL** → human classifies as EITHER:
  (i) **expected error path** you provoked intentionally — correct CLI behavior,
      fine; OR
  (ii) **critical bug** — the CLI broke when it should not have → STOP, report,
       do NOT release. File it (`carpenter bug file`) if asked.

Also show the **drafted behavioral note** for the atom; the human confirms or
edits it.

## Phase E — Release + cleanup (only after sign-off)
1. Author `#[test] fn <fn>_*` in `src/commands/<module>.rs` (model it on the
   module's existing tests). This is the one piece `--capture-example` cannot
   produce.
2. Finalize the atom's `TODO` note from Phase-C observations.
3. Strict verify → ship check:
   ```
   cargo xtask build            # strict gates now pass (atom + test exist)
   cargo xtask build --release
   ```
4. Tear down the sandbox via the CLI (never `rm` yourself):
   ```
   carpenter dev clean
   ```

**Deliverables kept**: `docs/examples/<module>/<fn>.md` + the test in
`src/commands/<module>.rs`. **Deleted**: the entire `.sandbox` (course DB, venv,
notebooks, config) — by `carpenter dev clean`, always, even on failure.

## Never
- Never call `rm`, `mkdir`, or `uv` directly — the CLI owns all of it.
- Never edit outside `docs/examples/**` and `src/commands/**`.
- Never proceed past Phase D without explicit human sign-off.
- Never hand-edit `src/howto.gen.md` or `docs/specs/*` generated regions — they
  regenerate via `cargo xtask build`.
