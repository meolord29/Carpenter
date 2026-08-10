# CourseSpec

Authored JSON consumed by `course create --spec -` / `course update --spec -`.

<!-- BEGIN GENERATED -->
| field | type | rule |
|-------|------|------|
| slug | string? | derived from title if absent ([conventions](../data-model/02-conventions.md#slug-derivation)) |
| title | string | required, non-empty |
| goal | string | required, non-empty |
| description | string | optional, default `""` |

Example:

```json
{"title":"Data Structures","slug":"data-structures","goal":"Understand core data structures from the ground up","description":"…"}
```
<!-- END GENERATED -->
