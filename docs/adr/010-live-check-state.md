# ADR-010: live check state via pass_or_fail (no attempts table)

Date: 2026-08-09 · Status: Accepted

## Context
Earlier drafts had an append-only `attempts` table recording every `quiz run`
(`{ts, passed, total, rate, per_case}`) and derived lesson `complete` from "the
latest attempts row." Two problems:
1. Practice checks were helper-only and **unrecorded** (`helper.py` opened the DB
   read-only), yet status derivation required practice to have passing attempts — a
   direct contradiction (lessons containing practice could never be `complete`).
2. `attempts` was the only thing consuming a write path and a history model, while
   the product actually needs *current state* (is this quiz passing right now?), not
   score-over-time.

## Decision
- **Drop the `attempts` table.** Add live-state columns to `practice` and `quizzes`:
  `pass_or_fail INTEGER` and `last_check TEXT` (JSON `{passed,total,cases:[…]}`).
- `helper.py` opens the DB **read-write** and performs exactly one constrained
  `UPDATE … SET pass_or_fail=?, last_check=? WHERE id=?` after each check. It never
  inserts, deletes, or touches rows it did not just read.
- Status derives from `pass_or_fail` (+ `skip`, see adr/011) — see
  [data-model/04](../data-model/04-status-derivation.md). There is no history;
  `last_check` is a current snapshot, overwritten each run. `quiz results` reads it.
- `quiz run` is `nbconvert --execute --allow_errors`; the in-notebook helper cells do
  the scoring and write the columns. Scaffolding errors (detected via `scaffold_hash`)
  escalate; learner errors are scored as fails.
- **Migrations:** the initial schema ships as one `CREATE`-based migration. A full
  versioned `migrate` command (`user_version`-based, fail-closed on future versions)
  is deferred until the first post-ship schema change (acceptable pre-1.0 with no
  users).

## Consequences
+ Resolves the practice/unrecorded contradiction: practice now records `pass_or_fail`
  just like quizzes, via the same helper write.
+ One write path (the helper's `UPDATE`), one source of current status.
+ `quiz run` needs no custom subprocess child — it reuses nbconvert + the helper,
  eliminating a second execution path and a second security boundary.
− No attempt history / score-over-time. Accepted: live state is the product need;
  `last_check` covers "what happened on the last run."
− `helper.py` is no longer read-only — but the write surface is one constrained
  `UPDATE`; the safety property is "never `expected` to stdout," preserved.
− Schema changes before the `migrate` command exists force users to rebuild
  `course.db`. Accepted pre-1.0.
