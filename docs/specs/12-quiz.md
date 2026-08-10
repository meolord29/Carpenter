# `quiz` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `run [lesson_id] [--timeout 30]` | — | `{"lesson_id":"arrays-101","quizzes":[{"quiz_id":"q1","skipped":false,"pass_or_fail":true,"passed":1,"total":1,"cases":[{"case_id":"c1","passed":true}]}],"saved":true}` — nbconvert in the course venv; `ExecuteError` on scaffolding errors; `StoreError` if no venv |
| `list [lesson_id]` | — | `{"quizzes":[{"id":"q1","lesson_id":"arrays-101","name":"max_value","case_count":1,"skip":false,"pass_or_fail":false}]}` |
| `show <quiz_id>` | — | `{"id":"q1","lesson_id":"arrays-101","name":"max_value","signature":"def max_value(arr):","prompt":"…","cases":1,"skip":false,"pass_or_fail":false}` |
| `results <quiz_id>` | — | `{"quiz_id":"q1","skipped":false,"pass_or_fail":true,"passed":1,"total":1,"cases":[{"case_id":"c1","passed":true}]}` — live snapshot from `last_check` (no history) |
<!-- END GENERATED -->
