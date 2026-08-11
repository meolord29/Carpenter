**example:**

```sh
carpenter lesson new --out lesson.yaml
```

No course context required (`-c` omitted). Prints a YAML template to stdout by
default; `--out <FILE>` writes it. Block scalars (`|`) keep multi-line
`content`/`solution` as-is (no `\n` escaping); signatures are quoted (they end
in `:`).
```yaml
title: <lesson title>
slug: <lesson-slug>          # optional; derived from title if omitted
sections:
  - title: <section title>
    snippets:
      - kind: markdown
        content: |
          ## Heading
          Prose here.
    practice:
      - name: <fn_name>
        signature: "def <fn_name>(x):"
        solution: |
          def <fn_name>(x):
              return x
        cases:
          - compare: exact
            args: [1]
            expected: 1
quizzes: []
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"lesson template written: lesson.yaml","data":{"written_to":"lesson.yaml"}}
```

Edit the template, then `carpenter -c <slug> lesson verify --spec lesson.yaml`
to lock the keys, and `carpenter -c <slug> lesson create --spec lesson.yaml` to
render it. Print mode (`carpenter lesson new`) returns the YAML in
`data.yaml` instead of `data.written_to`.
