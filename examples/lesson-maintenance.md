# Maintain a lesson after learner edits

The authoring maintenance loop: push a content fix to a lesson that learners
have already edited, without destroying their stub work, then re-verify the
answer key and re-score. Running example: `arrays-101` in the `ds` course.

The fenced ` ```sh ` blocks below are the real flow; the ` ```yaml ` blocks are
the specs and the ` ```json ` blocks are the result envelopes. (Only the `sh`
blocks are counted by the compile-time scenario gate — see
`docs/adr/013-compile-enforced-scenarios.md`.)

## 1. Update the lesson (destructive — requires --force)

`lesson update` replaces the DB content and re-renders the notebook. `--force`
is the confirmation that you accept the rewrite.

```sh
carpenter -c ds lesson update arrays-101 --spec <lesson-spec>.yaml --force
```
```json
{"status":"ok","message":"lesson updated: arrays-101","data":{"id":"arrays-101","updated":{"id":"arrays-101","slug":"arrays-101","title":"Arrays 101","ord":1,"status":"in_progress","skip":false,"created_at":"2026-08-09T12:00:00Z","updated_at":"2026-08-10T09:00:00Z"}}}
```

## 2. Sync learner edits back (3-way stub preservation)

If the notebook was updated out-of-band (or you want the DB state re-rendered
without clobbering learner stub edits), `lesson sync` performs a 3-way merge:
learner edits to stub cells are kept; anything unresolvable surfaces in
`conflicts[]`. A clean sync needs no `--force`.

```sh
carpenter -c ds lesson sync arrays-101
```
```json
{"status":"ok","message":"lesson synced: arrays-101","data":{"id":"arrays-101","synced":true,"conflicts":[]}}
```

## 3. Re-verify the answer key

`lesson verify <id>` re-runs the stored author solutions against their own
cases in the course venv — after an update, this proves the new key is
self-consistent before any learner sees it (adr/015).

```sh
carpenter -c ds lesson verify arrays-101
```
```json
{"status":"ok","message":"lesson verified: arrays-101","data":{"lesson_id":"arrays-101","checked":2,"passing":2,"failing":0,"checkables":[{"owner_type":"practice","owner_id":"p1","name":"sum_array","has_solution":true,"passed":1,"total":1,"cases":[{"case_id":"c1","passed":true}]},{"owner_type":"quiz","owner_id":"q1","name":"max_value","has_solution":true,"passed":1,"total":1,"cases":[{"case_id":"c2","passed":true}]}]}}
```

## 4. Re-execute and re-score

Confirm the teaching cells still run clean, then score the notebooks.

```sh
carpenter -c ds lesson execute arrays-101 --allow-errors
```
```json
{"status":"ok","message":"lesson executed: arrays-101","data":{"id":"arrays-101","executed":true,"cells":{"total":7,"ran":7,"errored":0},"errors":[]}}
```

```sh
carpenter -c ds quiz run arrays-101
```
```json
{"status":"ok","message":"quizzes run: arrays-101","data":{"lesson_id":"arrays-101","quizzes":[{"quiz_id":"q1","skipped":false,"pass_or_fail":false,"passed":0,"total":1,"cases":[{"case_id":"c2","passed":false,"error":"NotImplementedError: "}]}],"saved":true}}
```

## Conventions (maintenance discipline)

- Never hand-edit a rendered `lesson.ipynb` — regenerate via `lesson update` /
  `lesson sync`.
- Notebook execution is serialized per course (adr/017): a concurrent
  `lesson execute`/`quiz run` waits on an execution lock (up to 120 s), so
  step 4's two commands run in order even if issued together.
- After a content change, always end the loop at `quiz run`: fresh state is
  all quizzes `pass_or_fail:false` until learners re-attempt.
