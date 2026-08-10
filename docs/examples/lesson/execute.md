**example:**

```sh
carpenter -c ds lesson execute arrays-101 --timeout 30 --allow-errors
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson executed: arrays-101","data":{"id":"arrays-101","executed":true,"cells":{"total":3,"ran":3,"errored":0},"errors":[]}}
```

Requires the course venv (`carpenter venv create`) else StoreError. `--allow-errors` runs every cell and returns all errors instead of aborting on the first.
