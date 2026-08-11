# `build` / `install` / `upgrade` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `build <path>` | target dir | `{"path":"/courses/ds","slug":"ds","created":["course.json","course.db","lessons/"]}` — scaffolds course.json + course.db + lessons/ |
| `install [--bin-dir <p>]` | — | `{"installed":true,"bin":"~/.local/bin/carpenter","on_path":true}` — `on_path` = whether `bin_dir` resolves on `$PATH` |
| `upgrade [--source <p>] [--bin-dir <p>] [--no-skill]` | source dir (`--source` → config `source_dir` → error) | `{"upgraded":true,"version":"0.5.0","bin":"~/.local/bin/carpenter","source":"/src/carpenter","skill":{"app":"opencode","path":"~/.config/opencode/skills/carpenter/SKILL.md","refreshed":true}}` — `skill` outcomes: `{refreshed:true,…}` · `{refreshed:false,reason:"not_registered",warning:"…"}` · `--no-skill` ⇒ `skill:null` |
<!-- END GENERATED -->

`upgrade` source resolves `--source` → `config source_dir` → `ValidationError`
(the user does the `git pull`). After replacing the binary it best-effort
refreshes a registered skill; `--no-skill` skips it (`skill:null`). The
`not_registered` warning string is single-sourced here; see
[adr/004](../adr/004-build-install-split.md).
