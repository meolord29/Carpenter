**example:**

```sh
carpenter -c ds venv sync
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"venv synced: ds","data":{"course":"ds","synced":true}}
```

Runs `uv sync` against the course `pyproject.toml`.
