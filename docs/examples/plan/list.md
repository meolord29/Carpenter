**example:**

```sh
carpenter -c ds plan list
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"plans listed","data":{"plans":[{"id":"pl1","scope":"course","scope_id":"ds","title":"…","confirmed":false}]}}
```

Optional `--scope course|lesson` filter.
