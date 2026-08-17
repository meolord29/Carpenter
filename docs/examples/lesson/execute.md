**example:**

```sh
carpenter -c ds lesson execute arrays-101 --timeout 30 --allow-errors
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson executed: arrays-101","data":{"id":"arrays-101","executed":true,"cells":{"total":3,"ran":3,"errored":0},"errors":[]}}
```

Requires the course venv (`carpenter venv create`) else StoreError. `--allow-errors` runs every cell and returns all errors instead of aborting on the first. Check cells run during execution and write live practice `pass_or_fail` (stdout shows `PASS`/`FAIL <id> <case>`, never the expected value) — practice is scored by executing, quizzes by `quiz run`. Notebook execution is serialized per course (adr/017): a concurrent `lesson execute`/`quiz run` in the same course waits (up to 120 s) on an execution lock, then fails with a clear StoreError.
