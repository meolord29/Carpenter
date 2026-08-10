**example:**

```sh
carpenter -c ds plan delete pl1 --force
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"plan deleted: pl1","data":{"id":"pl1","deleted":true}}
```

`--force` required only if the plan is confirmed.
