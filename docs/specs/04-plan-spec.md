# PlanSpec

Authored JSON consumed by `plan create --scope course|lesson --spec -`.

<!-- BEGIN GENERATED -->
| field | type | rule |
|-------|------|------|
| title | string | required |
| goals | string[] | bullet goals; become `goals` rows on `confirm` (course scope) |
| links | `{goal_index_<i>: lesson_id[]}` | maps each goal to covering lessons. `<i>` is the 0-based index into `goals[]` (range-checked at `create`; lesson ids resolved at `confirm`). A goal absent from `links` gets `covered_by:[]`. |

Example:

```json
{"title":"Data Structures — course plan","goals":["Know array/list internals","Implement a hash map from scratch"],"links":{"goal_index_0":["arrays-101","hashing-101"]}}
```
<!-- END GENERATED -->
