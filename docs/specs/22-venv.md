# `venv` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `create [--python 3.12]` | python version | `{"course":"ds","python":"3.12","path":"<root>/courses/ds/.venv","deps":["jupyterlab","nbconvert","nbclient","ipykernel"]}` — `StoreError` if no uv; `AlreadyExists` if `.venv` present |
| `sync` | — | `{"course":"ds","synced":true}` |
| `list` | — | `{"course":"ds","packages":[{"name":"nbconvert","version":"7.16.4"}]}` |
| `add <pkg>` | package name (repeatable) | `{"course":"ds","added":["numpy"],"packages":[{"name":"numpy","version":"2.1.3"}]}` |
<!-- END GENERATED -->
