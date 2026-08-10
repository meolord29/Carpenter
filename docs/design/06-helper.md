# Self-check helper (verification-only)

## Generation
On `lesson create` / `lesson sync`, `core/helper.rs` writes `helper.py` into the
lesson dir (`courses/<slug>/lessons/<NN-slug>/helper.py`), alongside the notebook.
It is **generic** — identical logic for every lesson, and it **never embeds cases**.
It opens the course DB and reads cases by id at runtime.

## Linking — by stable id
The link is the **id**, not a path or a copy. At render time `core/notebook.rs`
reads each practice/quiz row and emits a check cell with that row's id + function
name baked in:
```python
import helper
helper.check("practice", "p1", sum_array)   # "p1" == practice.id == test_cases.owner_id
```
At runtime `check(owner_type, owner_id, fn)` queries
`SELECT args, kwargs, expected, compare FROM test_cases WHERE owner_type=? AND owner_id=? ORDER BY ord`.
Since carpenter writes both the DB row and the check-cell source, they can't drift;
`sync` re-renders cells from the DB. Cases live in one place — change them via
`lesson update --spec` and the next check picks it up; `helper.py` needs no regen
for case changes.

## Path resolution
`helper.py` lives at `courses/<slug>/lessons/<NN-slug>/helper.py`; `course.db` is
two levels up. Resolved from `__file__` (cwd-independent):
```python
from pathlib import Path
_DB = Path(__file__).resolve().parents[2] / "course.db"   # courses/<slug>/course.db
```
Opened read-write: `sqlite3.connect(str(_DB))`. The connection performs **only** a
single constrained `UPDATE` after a check (below); it never deletes, inserts, or
touches rows it did not just read.

## Behaviour
`check` loads the owner's cases, calls `fn(*args, **kwargs)` per case, compares via
the embedded `_compare`, prints `PASS`/`FAIL` per case + a `k/n` summary. **Never
prints `expected`.** Stdlib `sqlite3` + `json` only — learner needs no deps. Compare
semantics match `core/compare.rs` (parity-tested).

After scoring, `check` writes the result back so status derivation has live state
(adr/010 — the `attempts` table is gone; this is the single source of current
status):
```sql
UPDATE <practice|quizzes>
   SET pass_or_fail = ?,           -- 1 iff all cases passed
       last_check   = ?            -- JSON {passed,total,cases:[{case_id,passed,error?}]}
 WHERE id = ?;
```
Per-case errors are caught (a runtime exception in `fn` ⇒ that case `passed=0,
error="…"`); an unfilled stub raises `NotImplementedError` ⇒ all cases fail, but the
check does not crash. If `fn` cannot be loaded at all (the cell raised on import),
`check` records nothing and the notebook execution surfaces that cell error to
nbconvert — carpenter then classifies it via `scaffold_hash` (see
[08-quiz-run.md](08-quiz-run.md)).
