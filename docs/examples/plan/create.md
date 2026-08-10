**example:**

```sh
carpenter -c ds plan create --scope course --spec plan.json
```

Input spec (`--spec <FILE|->`):
```json
{
  "title": "Data Structures — course plan",
  "goals": [
    "Know array/list internals",
    "Implement a hash map from scratch"
  ],
  "links": {
    "goal_index_0": [
      "arrays-101"
    ],
    "goal_index_1": [
      "hashing-101"
    ]
  }
}
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"plan draft created: pl1","data":{"id":"pl1","scope":"course","scope_id":"ds","title":"…","content":"{goals, links}","confirmed":false}}
```

`links` keys MUST be `goal_index_<i>` where `<i>` is the 0-based index into `goals[]`; a goal absent from `links` gets `covered_by:[]`. `--scope lesson --lesson <id>` scopes a plan to one lesson.
