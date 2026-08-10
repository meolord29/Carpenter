**example:**

```sh
carpenter config set timeout_secs 45
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"config set: timeout_secs","data":{"key":"timeout_secs","value":45}}
```

Value is coerced to the key's type.
