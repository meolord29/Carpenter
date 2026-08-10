**example:**

```sh
carpenter bug file --spec bug.json
```

Input spec (`--spec <FILE|->`):
```json
{
  "title": "quiz run ignores --timeout",
  "description": "The timeout flag has no effect.",
  "repro": "carpenter quiz run 01 …"
}
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"issue filed: b1","data":{"id":"b1","path":"/…/bug/b1.json","status":"open"}}
```

Bug-only: `repro` allowed, `rationale` ⇒ ValidationError. File-backed under ~/.config/carpenter/bug/.
