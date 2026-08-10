# Helper verification contract (in-notebook)

`helper.py` is generated per lesson (`courses/<slug>/lessons/<NN-slug>/helper.py`).
`check(owner_type, owner_id, fn)`:
- resolves `course.db` two levels up: `Path(__file__).resolve().parents[2] / "course.db"`, opened **read-write** (the only write is one constrained `UPDATE` after scoring — adr/010);
- reads `test_cases` for `owner_type`/`owner_id` via stdlib `sqlite3` + `json` (learner needs no deps);
- for each case: `got = fn(*args, **kwargs)`, then compare per `compare` mode (runtime exceptions in `fn` are caught ⇒ that case `passed=0, error="…"`; an unfilled stub raising `NotImplementedError` ⇒ all cases fail, no crash);
- prints `PASS`/`FAIL` per case and a summary `k/n`;
- **never prints `expected`** (verification-only);
- after scoring, writes the result back so status derivation has live state:
  `UPDATE <practice|quizzes> SET pass_or_fail=?, last_check=? WHERE id=?` — `pass_or_fail=1` iff all cases passed, `last_check` is JSON `{passed,total,cases:[{case_id,passed,error?}]}`.

Compare modes (must match `core/compare.rs`): `exact` `==`; `sorted`
`sorted(a)==sorted(b)`; `set` `set(a)==set(b)`. Edge cases both impls must agree on:
unsortable element under `sorted` ⇒ the case errors (`error:"unsortable"`); unhashable
element under `set` ⇒ `error:"unhashable"`; `NaN != NaN` (standard float rule).
