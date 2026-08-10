**example:**

```sh
carpenter upgrade --source /src/carpenter --bin-dir ~/.local/bin
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"upgraded: 0.1.0","data":{"upgraded":true,"version":"0.1.0","bin":"/home/u/.local/bin/carpenter","source":"/src/carpenter","skill":{"refreshed":true,"app":"opencode","path":"/home/u/.config/opencode/skills/carpenter/SKILL.md"}}}
```

Rebuilds from source, replaces the binary, and re-renders the registered skill. `--no-skill` skips the skill refresh (`skill:null`).
