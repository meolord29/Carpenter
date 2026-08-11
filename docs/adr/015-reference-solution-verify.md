# ADR-015: reference `solution` field + `lesson verify`

Date: 2026-08-11 · Status: Accepted

## Context

Each practice/quiz carries test `cases` with `expected` values (the answer key),
but carpenter had **no reference implementation** to check those keys against. An
author/agent had to "lock the keys" with a throwaway standalone Python script
that implements each fn and asserts under `==` — a convention documented in
`examples/build-a-course.md` ("Conventions") and the skill's walkthrough. That
worked, but:

- it lived **outside the CLI** (an agent authored `verify_l4.py` per lesson);
- the throwaway script's compare logic could drift from carpenter's
  (`core/compare.rs` / `helper.py`'s `_compare`);
- the key was re-derived by hand each time, never stored, never re-checkable
  after a spec edit.

## Decision

Add an optional **author reference `solution`** to each `CheckableSpec`
(`practice[]`/`quizzes[]`): Python source that defines the fn named `name`. Add
**`lesson verify`**, which runs each solution against its own cases and reports
per-case pass/fail.

- **Field:** `#[serde(default, skip_serializing_if = "Option::is_none")] solution:
  Option<String>` on `CheckableSpec`. Optional (old specs unchanged; absent from
  serialization when `None`).
- **Storage:** nullable `solution TEXT` on `practice` + `quizzes`, back-filled on
  existing DBs by the first idempotent migration in `core/db.rs::migrate`
  (gated on `PRAGMA table_info`, since SQLite lacks `ADD COLUMN IF NOT EXISTS`).
- **Execution:** a new generated **`verify.py`** (stdlib-only), staged in a temp
  dir and run via `uv run` in the **course venv with a per-case timeout**. It
  reuses the same `_compare` as `helper.py`, so the compare-parity invariant
  holds. Author code runs in the subprocess (the isolation boundary) — same as
  learner code in `quiz run`.
- **Two modes:** `lesson verify --spec <FILE|->` (pre-create key-lock, replaces
  the throwaway script; `lesson_id:null`, `owner_id` = fn `name`) and
  `lesson verify <id>` (re-verify stored solutions after edits; `owner_id` =
  `p1`/`q1`…).
- **Invariants preserved:**
  - `helper.py` is **byte-untouched** — its verification-only contract
    (`helper.rs:1-5`, invariant test `helper.rs:118-120`) is unaffected; verify
    is a separate artifact/path.
  - The `solution` is **author-only**: never rendered into the notebook, absent
    from learner-facing output trees (`CheckableTree`). Only `lesson verify`
    reads it.
  - **Never expose `expected`:** the verify envelope carries `passed` + `actual`
    (`repr`) + `error` per case — never `expected`. The absolute invariant from
    [adr/010](010-live-check-state.md) holds on this path too.

## Consequences

+ The documented "throwaway script" convention becomes a first-class command;
  the key is locked by the **same compare logic** that grades the learner, so
  author and learner grading cannot drift.
+ `--spec` mode lets an author lock keys **before** `lesson create` — the agent's
  actual workflow, now in-CLI.
+ The reference solution is persisted, so `lesson verify <id>` re-checks anytime
  (e.g. after `lesson update` changes cases).

− A schema change (new column) + the project's first runtime migration. The
  migration is idempotent and tested-by-construction (`has_column` guard).
− carpenter now runs **author** Python at authoring time, in addition to learner
  Python at `quiz run`/`lesson execute` time. Same trust boundary (course venv,
  isolated subprocess, timeout); authoring-time author code is no less trusted
  than the spec itself.
− Per-case timeout uses `signal.SIGALRM` (Unix); on Windows there is no hard kill
  (best-effort), matching nbconvert's cross-platform behavior.

## Rejected

- **`--verify` on `lesson create`** (auto-lock at create) — deferred (YAGNI);
  ship standalone `lesson verify` first. Easy to add later.
- **Embedding the reference solution in `helper.py` / the notebook** — would
  break the helper's "identical for every lesson / verification-only / never
  embeds cases" contract and risk leaking the answer key to the learner.
- **Executing the reference solution in Rust** (PyO/inline) — would pull Python
  into carpenter's process, violating the subprocess isolation boundary.
