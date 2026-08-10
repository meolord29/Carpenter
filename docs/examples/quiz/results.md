**example:**

```sh
carpenter -c ds quiz results q1
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"quiz results: q1","data":{"quiz_id":"q1","skipped":false,"pass_or_fail":true,"passed":1,"total":1,"cases":[{"case_id":"c1","passed":true}]}}
```

Last-check results (from the most recent `quiz run`).
