# `notes` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `add --spec -` | NoteSpec | `{"id":"n1","kind":"gap","tags":["recursion"],"status":"open","recurrence":"new","related":"q2","text":"Learner struggles with base cases.","related_open":[]}` — `related_open` = open notes sharing ≥1 tag (excluding self) — advisory; `recurrence` is never auto-changed |
| `show <id>` | — | `{"notes":[{"id":"n1","kind":"gap","tags":["recursion"],"status":"open","recurrence":"new","related":"q2","text":"Learner struggles with base cases."}]}` — `NotFound` if absent |
| `list` | — | `{"notes":[{"id":"n1","kind":"gap","tags":["recursion"],"status":"open","recurrence":"new","related":"q2","text":"Learner struggles with base cases."}],"errors":[]}` — corrupt rows surface in `errors[]` |
| `update <id> --spec -` | NoteSpec | `{"id":"n1","updated":{"id":"n1","kind":"gap","tags":["recursion"],"status":"open","recurrence":"new","related":"q2","text":"Learner struggles with base cases."}}` |
| `resolve <id>` | — | `{"id":"n1","status":"resolved"}` |
| `remove <id> --force` | — | `{"id":"n1","deleted":true}` — `Conflict` without `--force` |
<!-- END GENERATED -->

`recurrence` is owned by the author ([06-note-spec.md](06-note-spec.md));
`related_open` is computed, read-only, and advisory.
