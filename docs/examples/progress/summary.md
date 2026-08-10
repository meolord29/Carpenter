**example:**

```sh
carpenter -c ds progress summary
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"progress summarized","data":{"lessons":{"total":1,"complete":0,"in_progress":1,"skipped":0},"quizzes":{"passing":1,"total":1},"goals":{"total":1,"achieved":0},"notes":{"total":1,"open":1,"recurring":0,"by_kind":{"gap":1,"mistake":0,"strength":0,"pattern":0,"progress":0}}}}
```
