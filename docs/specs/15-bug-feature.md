# `bug` / `feature` — output contracts

Identical shape; `bug` writes `~/.config/carpenter/bug/`, `feature` writes
`feature_request/`.

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `file --spec -` | IssueSpec | `{"id":"b1","path":"~/.config/carpenter/bug/b1.json","status":"open"}` — `id` is `<prefix><N>` (`b1`… for bug; `f1`… for feature), `max+1` per kind |
| `list` | — | `{"items":[{"id":"b1","title":"quiz run ignores --timeout","status":"open"}],"errors":[]}` — corrupt files surface in `errors[]` |
| `show <id>` | — | `{"id":"b1","title":"quiz run ignores --timeout","description":"The timeout flag has no effect.","repro":"carpenter quiz run 01 …","rationale":null,"status":"open","resolved_ts":null}` — `NotFound` if absent |
| `resolve <id>` | — | `{"id":"b1","status":"resolved","resolved_ts":"2026-08-09T12:00:00Z"}` |
<!-- END GENERATED -->
