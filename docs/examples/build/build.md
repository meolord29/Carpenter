**example:**

```sh
carpenter build /courses/ds
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"course built: ds","data":{"path":"/courses/ds","slug":"ds","created":["course.json","course.db","lessons/"]}}
```

Low-level scaffold (no spec). Prefer `course create`.
