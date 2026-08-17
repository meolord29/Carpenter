# Tutor feedback loop

The live-tutoring half of carpenter: score a learner's quizzes, inspect the
last check, exclude a broken quiz from status derivation, record the gap as a
note, and roll up progress. Running example: an `arrays-101` lesson in the
`ds` course that the learner has partially attempted.

The fenced ` ```sh ` blocks below are the real flow; the ` ```yaml ` blocks are
the specs and the ` ```json ` blocks are the result envelopes. (Only the `sh`
blocks are counted by the compile-time scenario gate — see
`docs/adr/013-compile-enforced-scenarios.md`.)

## 1. Score the quizzes

`quiz run` executes the notebook in the course venv; helper cells write live
`pass_or_fail` per quiz. A wrong learner answer fails its case (never showing
the expected value).

```sh
carpenter -c ds quiz run arrays-101
```
```json
{"status":"ok","message":"quizzes run: arrays-101","data":{"lesson_id":"arrays-101","quizzes":[{"quiz_id":"q1","skipped":false,"pass_or_fail":true,"passed":1,"total":1,"cases":[{"case_id":"c1","passed":true}]},{"quiz_id":"q2","skipped":false,"pass_or_fail":false,"passed":0,"total":2,"cases":[{"case_id":"c2","passed":false,"error":"AssertionError: "},{"case_id":"c3","passed":false,"error":"NotImplementedError: "}]}],"saved":true}}
```

## 2. Inspect the last check

`quiz results` replays the most recent run's saved state without re-executing.

```sh
carpenter -c ds quiz results q2
```
```json
{"status":"ok","message":"quiz results: q2","data":{"quiz_id":"q2","skipped":false,"pass_or_fail":false,"passed":0,"total":2,"cases":[{"case_id":"c2","passed":false,"error":"AssertionError: "},{"case_id":"c3","passed":false,"error":"NotImplementedError: "}]}}
```

## 3. Park a broken quiz

`c3` fails with `NotImplementedError` — the learner never attempted it. Skip
`q2` for now so lesson status reflects attempted work only; skipped items are
excluded from status derivation.

```sh
carpenter -c ds skip q2 --scope quiz
```
```json
{"status":"ok","message":"skip set: quiz q2","data":{"scope":"quiz","id":"q2","skip":true}}
```

Later, `--off` re-includes it:

```sh
carpenter -c ds skip q2 --scope quiz --off
```
```json
{"status":"ok","message":"skip cleared: quiz q2","data":{"scope":"quiz","id":"q2","skip":false}}
```

## 4. Record the gap

`notes add` captures what the tutor observed, linked to the failing quiz
(`related_open` surfaces recurring gaps on future notes).

```sh
carpenter -c ds notes add --spec <note-spec>.yaml
```

`<note-spec>.yaml`:
```yaml
kind: gap
tags:
  - base-cases
recurrence: new
related: q2
text: Learner attempts recursion but leaves the base case unimplemented under time pressure.
```
```json
{"status":"ok","message":"note added: n1","data":{"id":"n1","kind":"gap","tags":["base-cases"],"status":"open","recurrence":"new","related":"q2","text":"Learner attempts recursion but leaves the base case unimplemented under time pressure.","related_open":[]}}
```

```sh
carpenter -c ds notes list
```
```json
{"status":"ok","message":"notes listed","data":{"notes":[{"id":"n1","kind":"gap","tags":["base-cases"],"status":"open","recurrence":"new","related":"q2","text":"…","related_open":[]}],"errors":[]}}
```

## 5. Roll up progress

```sh
carpenter -c ds progress show
```
```json
{"status":"ok","message":"progress shown","data":{"lessons":[{"id":"arrays-101","title":"Arrays 101","status":"in_progress","skip":false,"passing":1,"total":2}]}}
```

```sh
carpenter -c ds progress summary
```
```json
{"status":"ok","message":"progress summarized","data":{"lessons":{"total":1,"complete":0,"in_progress":1,"skipped":0},"quizzes":{"passing":1,"total":2},"goals":{"total":1,"achieved":0},"notes":{"total":1,"open":1,"recurring":0,"by_kind":{"gap":1,"mistake":0,"strength":0,"pattern":0,"progress":0}}}}
```

## Conventions (tutor discipline)

- Resolve the note when the gap closes (`notes resolve n1`) — `progress
  summary` counts open notes by kind.
- A goal derives `achieved` only when every `covered_by` lesson is `complete`;
  skipping is the honest way to let a blocked lesson stop counting.
