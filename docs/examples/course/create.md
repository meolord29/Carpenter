**example:**

```sh
carpenter course create --spec course.json
```

Input spec (`--spec <FILE|->`):
```json
{
  "title": "Data Structures",
  "slug": "data-structures",
  "goal": "Understand core data structures from the ground up",
  "description": "Arrays, lists, hashing, trees."
}
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"course created: data-structures","data":{"slug":"data-structures","title":"Data Structures","path":"/…/courses/data-structures"}}
```
