**example:**

```sh
carpenter -c ds goal update g1 --status achieved --covered-by hashing-101
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"goal updated: g1","data":{"id":"g1","status":"achieved","override":true,"covered_by":["hashing-101"]}}
```

`--status derived` resumes live derivation; `--covered-by` is comma-separated.
