# Compare

`compare(actual, expected, mode)`:
- `exact` — `actual == expected`.
- `sorted` — `sorted(actual) == sorted(expected)`.
- `set` — `set(actual) == set(expected)`.

The Rust impl lives in `core/compare.rs`; the helper embeds a Python impl with
identical semantics (locked in
[specs/20-helper-contract.md](../specs/20-helper-contract.md), asserted by parity
tests in each language).

## Edge cases (both impls must agree)
- `sorted` on an element that is not mutually sortable (e.g. mixed `int`/`dict`)
  ⇒ the case errors (`error:"unsortable"`), not a crash.
- `set` on an unhashable element (e.g. `list`/`dict`) ⇒ `error:"unhashable"`.
- Float `NaN`: `NaN != NaN` (standard rule) — a case expecting `NaN` will not match.
- `expected` is JSON; the impls operate on the deserialized Python/Rust values.
