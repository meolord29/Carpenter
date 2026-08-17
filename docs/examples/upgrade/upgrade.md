**example:**

```sh
carpenter upgrade --bin-dir ~/.local/bin
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"upgraded: 0.5.0","data":{"upgraded":true,"version":"0.5.0","bin":"/home/u/.local/bin/carpenter","source":"https://github.com/meolord29/Carpenter/releases/download/edge/carpenter-x86_64-unknown-linux-musl.tar.gz","skill":{"refreshed":true,"app":"opencode","path":"/home/u/.config/opencode/skills/carpenter/SKILL.md"}}}
```

Fetches the GitHub `edge` release (checksum-verified), replaces the binary, and
re-registers the skill. `--source <path>` (or config `source_dir`) rebuilds from
a local checkout instead — that mode refreshes the skill only if registered.
`--no-skill` skips the skill write (`skill:null`).
