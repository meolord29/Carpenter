**example:**

```sh
carpenter course update data-structures --spec course.json --force
```

Input spec (`--spec <FILE|->`):
```json
{
  "title": "Data Structures",
  "slug": "data-structures",
  "goal": "Understand core data structures from the ground up",
  "description": "Arrays, lists, hashing, trees."
}
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"course updated: ds","data":{"slug":"data-structures","updated":{"slug":"data-structures","title":"…","goal":"…","description":"…","created_at":"2026-08-09T12:00:00Z"}}}
```
