**example:**

```sh
carpenter -c ds notes update n1 --spec note.yaml
```

Input spec (`--spec <FILE|->`):
```yaml
kind: gap
tags:
  - recursion
recurrence: new
related: q2
text: Learner struggles with base cases.
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"note updated: n1","data":{"id":"n1","updated":{"id":"n1","kind":"gap","tags":["recursion"],"status":"open","recurrence":"new","related":"q2","text":"Learner struggles with base cases."}}}
```
