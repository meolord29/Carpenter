# NoteSpec

Authored JSON consumed by `notes add --spec -` / `notes update --spec -`.

<!-- BEGIN GENERATED -->
| field | type | rule |
|-------|------|------|
| kind | enum | `gap\|mistake\|strength\|pattern\|progress` — required |
| tags | string[] | default `[]` |
| recurrence | enum | `new`(default) \| `recurring` — **authored**; the system never overwrites it (it may surface `related_open` as a hint in `add` output — see [14-notes.md](14-notes.md)) |
| related | string? | a lesson/quiz id; stored as free text (no FK) — an unresolvable id is kept as-is, not rejected |
| text | string | required, non-empty |

Example:

```json
{"kind":"gap","tags":["recursion"],"recurrence":"new","related":"q2","text":"Learner struggles with base cases."}
```
<!-- END GENERATED -->
