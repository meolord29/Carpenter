**example:**

```sh
carpenter -c ds plan update pl1 --spec plan.yaml
```

Input spec (`--spec <FILE|->`):
```yaml
title: "Data Structures — course plan"
goals:
  - Know array/list internals
  - Implement a hash map from scratch
links:
  goal_index_0:
    - arrays-101
  goal_index_1:
    - hashing-101
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"plan updated: pl1","data":{"id":"pl1","updated":{"id":"pl1","scope":"course","scope_id":"ds","title":"…","content":"{goals, links}","created_at":"2026-08-09T12:00:00Z","confirmed_at":null}}}
```

Only while unconfirmed.
