# Status derivation

Status is derived from `pass_or_fail` + `skip` columns (set by the helper on each
check). It is **never** manually set — there is no `mark-status` command. Skip is
the only learner/agent input that affects status beyond passing checks.

## Lesson status (`lessons.status`)
Computed on read from the lesson's non-skipped practice and quizzes. Order:

1. `lessons.skip = 1` ⇒ **`skipped`**.
2. Let `items` = practice + quizzes under the lesson with `skip = 0`.
   `items_passing` = those with `pass_or_fail = 1`.
3. `items` is empty ⇒ **`not_started`**.
4. `|items_passing| == |items|` ⇒ **`complete`** (all non-skipped items pass).
5. `|items_passing| > 0` ⇒ **`in_progress`**.
6. Else (some items exist, none passing) ⇒ **`not_started`**.

Skipped items (`skip = 1`) are excluded entirely — a lesson can be `complete` even
if a skipped quiz is unfilled. A lesson with zero non-skipped items is `not_started`
unless `lessons.skip = 1`.

`lessons.status` is a denormalized cache; it is refreshed whenever any child's
`pass_or_fail`/`skip` changes (helper writes `pass_or_fail`/`last_check`, `skip`
command flips the `skip` column) and may be recomputed on read.

## Goal status (`goals.status`)
1. `goals.override = 1` ⇒ the authored value (`pending|achieved|skipped`) wins;
   derivation is skipped. `goal update --status` sets `override = 1`.
2. Else: let `covered` = `covered_by` lessons.
   - `covered` is empty ⇒ **`pending`** (vacuous-completion is *not* achieved).
   - all `covered` lessons are `complete` ⇒ **`achieved`**.
   - otherwise ⇒ **`pending`**.
3. `goal update --status derived` clears `override = 0` to resume derivation.

## Edge cases
- Empty `covered_by` ⇒ `pending` (not auto-achieved).
- Empty lesson (no practice, no quizzes) ⇒ `not_started` unless skipped.
- A `complete` lesson whose last passing item later has `pass_or_fail` cleared
  (e.g. material edited so the check now fails) ⇒ reverts to `in_progress` or
  `not_started` on next derivation.

Rationale: the `attempts` table was removed (adr/010). Live state lives in
`pass_or_fail`/`last_check`; there is no attempt history and no manual override on
lessons — skip is the only escape hatch.
