**example:**

```sh
carpenter -c ds lesson verify --spec -
```

Input spec (`--spec <FILE|->`, JSON or YAML). Each Checkable may carry a
`solution` — Python source defining the fn named `name`. `lesson verify` runs
each solution against its own cases in the course venv (the answer-key lock).
```yaml
title: Arrays 101
slug: arrays-101
sections:
  - title: Intro
    snippets:
      - kind: markdown
        content: "# hi"
    practice:
      - name: sum_array
        signature: "def sum_array(arr):"
        solution: |
          def sum_array(arr):
              return sum(arr)
        cases:
          - compare: exact
            args:
              - [1, 2, 3]
            kwargs: {}
            expected: 6
quizzes: []
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson verified: (spec)","data":{"lesson_id":null,"checked":1,"passing":1,"failing":0,"checkables":[{"owner_type":"practice","owner_id":"sum_array","name":"sum_array","has_solution":true,"passed":1,"total":1,"cases":[{"case_id":"sum_array-0","passed":true}]}]}}
```

`--spec` verifies pre-create (`lesson_id:null`, `owner_id` = fn `name`); `<id>`
re-verifies stored solutions post-create (`owner_id` = `p1`/`q1`…). Requires
`carpenter venv create`. A checkable without a `solution` reports
`has_solution:false`; its cases fail with `error:"no solution"`. Results carry
`actual`/`error`, never `expected` (adr/015); `actual` is string-encoded
(e.g. `"actual":"0.25"`), so compare it textually, not as JSON numbers.
