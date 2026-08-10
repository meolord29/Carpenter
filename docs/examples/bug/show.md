**example:**

```sh
carpenter bug show b1
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"issue shown: b1","data":{"id":"b1","title":"quiz run ignores --timeout","description":"The timeout flag has no effect.","repro":"carpenter quiz run 01 …","rationale":null,"status":"open","resolved_ts":null}}
```
