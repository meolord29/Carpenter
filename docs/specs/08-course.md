# `course` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `create --spec -` | CourseSpec | `{"slug":"data-structures","title":"Data Structures","path":"<root>/courses/data-structures"}` — `AlreadyExists` on duplicate slug |
| `list` | — | `{"courses":[{"slug":"…","title":"…","goal":"…","lessons_count":0}],"errors":[]}` |
| `show <slug>` | — | `{"slug":"…","title":"…","goal":"…","description":"…","counts":{"lessons":0,"sections":0,"practice":0,"quizzes":0}}` — `NotFound` if absent |
| `update <slug> --spec - --force` | CourseSpec | `{"slug":"…","updated":{"slug":"…","title":"…","goal":"…","description":"…","created_at":"2026-08-09T12:00:00Z"}}` — `Conflict` without `--force` |
| `delete <slug> --force` | — | `{"slug":"…","deleted":true}` — `Conflict` without `--force` |
| `switch <slug>` | — | `{"active_course":"…"}` — writes config |
<!-- END GENERATED -->
