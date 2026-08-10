**example:**

```sh
carpenter -c ds venv add numpy pandas
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"venv added: ds","data":{"course":"ds","added":["numpy","pandas"],"packages":[{"name":"numpy","version":"2.1.3"}]}}
```

One or more package names, positional.
