**example:**

```sh
carpenter -c ds notes add --spec note.yaml
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
{"status":"ok","message":"note added: n1","data":{"id":"n1","kind":"gap","tags":["recursion"],"status":"open","recurrence":"new","related":"q2","text":"Learner struggles with base cases.","related_open":[]}}
```

`kind` ∈ gap|mistake|strength|pattern|progress. `related_open` is an advisory hint (recurring gaps).
