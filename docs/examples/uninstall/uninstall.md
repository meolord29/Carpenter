**example:**

```sh
carpenter uninstall --bin-dir ~/.local/bin
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"uninstalled: /home/u/.local/bin/carpenter","data":{"uninstalled":true,"bin":"/home/u/.local/bin/carpenter","skill":{"app":"opencode","path":"/home/u/.config/opencode/skills/carpenter/SKILL.md","removed":true},"config_purged":false}}
```

Inverse of `install`: removes the opencode skill (best-effort — `{"removed":false,"reason":"not_registered"}` when absent), then deletes `<bin_dir>/carpenter` (safe while running on Linux/macOS). `NotFound` when neither skill nor binary exists. `--purge-config` also removes the config file; course data is never touched.
