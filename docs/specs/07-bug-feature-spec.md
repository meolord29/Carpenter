# BugSpec / FeatureSpec

Authored JSON consumed by `bug file --spec -` / `feature file --spec -`. Identical
shape; `repro` is bug-only, `rationale` is feature-only.

<!-- BEGIN GENERATED -->
| field | type | rule |
|-------|------|------|
| title | string | required, non-empty |
| description | string | required, non-empty |
| repro | string? | bug only — passing it on a feature (or with `rationale`) ⇒ `ValidationError` |
| rationale | string? | feature only — passing it on a bug (or with `repro`) ⇒ `ValidationError` |

Example:

```json
{"title":"quiz run ignores --timeout","description":"The timeout flag has no effect.","repro":"carpenter quiz run 01 …","rationale":null}
```
<!-- END GENERATED -->
