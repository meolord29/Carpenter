**example:**

```sh
carpenter -c ds plan confirm pl1
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"plan confirmed: pl1","data":{"id":"pl1","confirmed":true,"confirmed_at":"2026-08-09T12:00:00Z","goals_created":["g1","g2"]}}
```

Course scope materializes one goal per `goals[]` entry (ids resolved now); lesson scope creates none. Confirming twice is a Conflict.
