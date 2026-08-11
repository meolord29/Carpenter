**example:**

```sh
carpenter -c ds goal add --spec goal.yaml
```

Input spec (`--spec <FILE|->`):
```yaml
text: Implement a hash map from scratch
covered_by:
  - hashing-101
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"goal added: g1","data":{"id":"g1","text":"Implement a hash map from scratch","status":"pending","covered_by":["hashing-101"]}}
```

`covered_by` lesson ids are resolved on use (unresolved ⇒ ValidationError).
