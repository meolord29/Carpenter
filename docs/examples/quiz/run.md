**example:**

```sh
carpenter -c ds quiz run arrays-101 --timeout 30
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"quizzes run: arrays-101","data":{"lesson_id":"arrays-101","quizzes":[{"quiz_id":"q1","skipped":false,"pass_or_fail":true,"passed":1,"total":1,"cases":[{"case_id":"c1","passed":true}]}],"saved":true}}
```

Requires the course venv. Writes live `pass_or_fail` per quiz; skipped quizzes report `skipped:true`. Notebook execution is serialized per course (adr/017): a concurrent `quiz run`/`lesson execute` in the same course waits (up to 120 s) on an execution lock, then fails with a clear StoreError.
