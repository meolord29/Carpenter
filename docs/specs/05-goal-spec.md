# GoalSpec

Authored YAML consumed by `goal add --spec -`.

<!-- BEGIN GENERATED -->
| field | type | rule |
|-------|------|------|
| text | string | required, non-empty — the bullet goal |
| covered_by | string[] | default `[]` — lesson ids covering this goal (resolved on use; unresolved ⇒ `ValidationError`) |

Example:

```yaml
text: Implement a hash map from scratch
covered_by:
  - hashing-101
```
<!-- END GENERATED -->
