**example:**

```sh
carpenter -c ds quiz list
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"quizzes listed","data":{"quizzes":[{"id":"q1","lesson_id":"arrays-101","name":"max_value","case_count":1,"skip":false,"pass_or_fail":false}]}}
```

Optional positional lesson id filters to one lesson.
