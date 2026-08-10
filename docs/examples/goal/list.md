**example:**

```sh
carpenter -c ds goal list
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"goals listed","data":{"goals":[{"id":"g1","text":"…","status":"pending","derived_status":"pending","covered_by":["hashing-101"]}]}}
```
