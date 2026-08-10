# LessonSpec

Authored JSON consumed by `lesson create --spec -` / `lesson update --spec -`.

<!-- BEGIN GENERATED -->
| field | type | rule |
|-------|------|------|
| title | string | required |
| slug | string? | derived from title if absent |
| order | int? | appended (max+1) if absent |
| sections[].snippets | `{kind, content}`[] | required; **`snippets[0].kind == "markdown"`**; each renders one cell |
| sections[].practice / quizzes | Checkable[] | array index ⇒ `ord` |
| cases[].compare | enum | `exact`(default) \| `sorted` \| `set` |
| cases[].args | array | default `[]` |
| cases[].kwargs | object | default `{}` |
| cases[].expected | any | required |

**Checkable** (shared): `{name, signature, prompt?, cases[]}`. `expected` for a `sorted`/`set` case must be sortable/hashable else the case errors (`error:"unsortable"`/`"unhashable"`).

Example:

```json
{"title":"Arrays 101","slug":"arrays-101","order":1,"sections":[{"title":"What is an array","snippets":[{"kind":"markdown","content":"An array stores items contiguously…"},{"kind":"code","content":"import numpy as np\nnp.array([1, 2, 3])"}],"practice":[{"name":"sum_array","signature":"def sum_array(arr):","prompt":"Return the sum of the array.","cases":[{"compare":"exact","args":[[1,2,3]],"kwargs":{},"expected":6},{"compare":"exact","args":[[]],"kwargs":{},"expected":0}]}]}],"quizzes":[{"name":"max_value","signature":"def max_value(arr):","prompt":"…","cases":[{"compare":"exact","args":[[3,1,2]],"kwargs":{},"expected":3}]}]}
```
<!-- END GENERATED -->
