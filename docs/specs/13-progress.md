# `progress` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `show` | — | `{"lessons":[{"id":"arrays-101","title":"Arrays 101","status":"in_progress","skip":false,"passing":1,"total":2}]}` — live per-lesson state (`passing`/`total` over non-skipped practice+quiz) |
| `summary` | — | `{"lessons":{"total":1,"complete":0,"in_progress":1,"skipped":0},"quizzes":{"passing":1,"total":1},"goals":{"total":1,"achieved":0},"notes":{"total":1,"open":1,"recurring":0,"by_kind":{"gap":1,"mistake":0,"strength":0,"pattern":0,"progress":0}}}` — `notes.by_kind` is an object keyed by kind; no history (adr/010) |
<!-- END GENERATED -->

`recent_attempts` was removed with the `attempts` table (adr/010); there is no
history, only current `pass_or_fail` state. `notes.by_kind` is an object keyed by
kind.
