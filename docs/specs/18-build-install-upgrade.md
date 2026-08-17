# `build` / `install` / `upgrade` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `build <path>` | target dir | `{"path":"/courses/ds","slug":"ds","created":["course.json","course.db","lessons/"]}` — scaffolds course.json + course.db + lessons/ |
| `install [--bin-dir <p>]` | — | `{"installed":true,"bin":"~/.local/bin/carpenter","on_path":true}` — `on_path` = whether `bin_dir` resolves on `$PATH` |
| `upgrade [--source <p>] [--bin-dir <p>] [--no-skill]` | no flag → GitHub `edge` release; `--source` → config `source_dir` → local build | `{"upgraded":true,"version":"0.5.0","bin":"~/.local/bin/carpenter","source":"https://github.com/meolord29/Carpenter/releases/download/edge/carpenter-x86_64-unknown-linux-musl.tar.gz","skill":{"app":"opencode","path":"~/.config/opencode/skills/carpenter/SKILL.md","refreshed":true}}` — `skill` outcomes: `{refreshed:true,…}` · `{refreshed:false,reason:"not_registered",warning:"…"}` (source mode) · `--no-skill` ⇒ `skill:null` |
<!-- END GENERATED -->

`upgrade` mode resolves: `--source <p>` → config `source_dir` → **GitHub `edge`
release** (curl + `SHA256SUMS` verify + extract — the same pipeline
`scripts/install.sh` uses). Release mode always (re-)registers the skill
(installer parity); source mode best-effort refreshes only a registered skill
(`not_registered` warning otherwise). `--no-skill` skips both (`skill:null`).
The `not_registered` warning string is single-sourced here; see
[adr/004](../adr/004-build-install-split.md), [adr/016](../adr/016-upgrade-fetches-release.md).
