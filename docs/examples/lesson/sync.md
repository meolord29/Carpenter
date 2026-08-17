**example:**

```sh
carpenter -c ds lesson sync arrays-101 --force
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson synced: arrays-101","data":{"id":"arrays-101","synced":true,"conflicts":[]}}
```

3-way stub preservation: learner edits to stub cells are kept; conflicts surface in `conflicts[]`. `--force` is required only when the sync would discard learner edits (unresolved conflicts); a clean sync succeeds without it. A synced notebook differs cosmetically from a freshly rendered one (single-string `source` fields, cleared outputs) — both parse identically; never normalize one by hand.
