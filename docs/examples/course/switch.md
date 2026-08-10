**example:**

```sh
carpenter course switch data-structures
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"active course switched: ds","data":{"active_course":"data-structures"}}
```

Writes `active_course` to config so `-c` can be omitted later.
