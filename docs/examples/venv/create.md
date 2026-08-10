**example:**

```sh
carpenter -c ds venv create --python 3.12
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"venv created: ds","data":{"course":"ds","python":"3.12","path":"<root>/courses/ds/.venv","deps":["jupyterlab","nbconvert","nbclient","ipykernel"]}}
```

Required before `lesson execute` / `quiz run`. Uses `uv`.
