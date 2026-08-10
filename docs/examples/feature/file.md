**example:**

```sh
carpenter feature file --spec feature.json
```

Input spec (`--spec <FILE|->`):
```json
{
  "title": "add dark mode",
  "description": "Users ask for a dark theme.",
  "rationale": "frequent user request"
}
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"issue filed: f1","data":{"id":"f1","path":"/…/feature_request/f1.json","status":"open"}}
```

Feature-only: `rationale` allowed, `repro` ⇒ ValidationError. File-backed under ~/.config/carpenter/feature_request/.
