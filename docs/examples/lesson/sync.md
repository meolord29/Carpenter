**example:**

```sh
carpenter -c ds lesson sync arrays-101 --force
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson synced: arrays-101","data":{"id":"arrays-101","synced":true,"conflicts":[]}}
```

3-way stub preservation: learner edits to stub cells are kept; conflicts surface in `conflicts[]`.
