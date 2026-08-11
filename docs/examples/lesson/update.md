**example:**

```sh
carpenter -c ds lesson update arrays-101 --spec lesson.yaml --force
```

Input spec (`--spec <FILE|->`):
```yaml
title: Arrays 101
slug: arrays-101
sections:
  - title: What is an array
    snippets:
      - kind: markdown
        content: An array stores items contiguously…
      - kind: code
        content: |
          import numpy as np
          np.array([1, 2, 3])
    practice:
      - name: sum_array
        signature: "def sum_array(arr):"
        prompt: Return the sum of the array.
        cases:
          - compare: exact
            args:
              - [1, 2, 3]
            kwargs: {}
            expected: 6
quizzes:
  - name: max_value
    signature: "def max_value(arr):"
    prompt: Return the max.
    cases:
      - compare: exact
        args:
          - [3, 1, 2]
        kwargs: {}
        expected: 3
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson updated: arrays-101","data":{"id":"arrays-101","updated":{"id":"arrays-101","slug":"arrays-101","title":"…","ord":1,"status":"not_started","skip":false,"created_at":"2026-08-09T12:00:00Z","updated_at":"2026-08-09T12:00:00Z"}}}
```

Destructive — requires `--force`. Re-renders the notebook.
