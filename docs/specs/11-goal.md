# `goal` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `add --spec -` | GoalSpec | `{"id":"g1","text":"Implement a hash map from scratch","covered_by":["hashing-101"],"status":"pending"}` |
| `list` | — | `{"goals":[{"id":"g1","text":"…","status":"pending","derived_status":"pending","covered_by":["hashing-101"]}]}` |
| `update <id> [--status <S>] [--covered-by …]` | — | `{"id":"g1","status":"achieved","override":true,"covered_by":["hashing-101"]}` — `<S>` pins (`override=1`) or `derived` resumes |
| `remove <id> --force` | — | `{"id":"g1","deleted":true}` — `Conflict` without `--force` |
<!-- END GENERATED -->
