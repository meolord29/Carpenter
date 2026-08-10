**example:**

```sh
carpenter -c ds skip q1 --scope quiz
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"skip set: quiz q1","data":{"scope":"quiz","id":"q1","skip":true}}
```

`--scope lesson|quiz|practice`. `--off` clears the flag. Skipped items are excluded from status derivation.
