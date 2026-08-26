# `build` / `install` / `upgrade` / `uninstall` — output contracts

<!-- BEGIN GENERATED -->
| cmd | input | `data` (ok) |
|-----|-------|-------------|
| `build <path>` | target dir | `{"path":"/courses/ds","slug":"ds","created":["course.json","course.db","lessons/"]}` — scaffolds course.json + course.db + lessons/ |
| `install [--bin-dir <p>]` | — | `{"installed":true,"bin":"~/.local/bin/carpenter","on_path":true}` — `on_path` = whether `bin_dir` resolves on `$PATH` |
| `upgrade [--source <p>] [--bin-dir <p>] [--no-skill]` | no flag → GitHub `edge` release; `--source` → config `source_dir` → local build | `{"upgraded":true,"version":"0.7.0","bin":"~/.local/bin/carpenter","source":"https://github.com/meolord29/Carpenter/releases/download/edge/carpenter-x86_64-unknown-linux-musl.tar.gz","skill":[{"app":"opencode","path":"~/.config/opencode/skills/carpenter/SKILL.md","refreshed":true},{"app":"claude-code","path":"~/.claude/skills/carpenter/SKILL.md","refreshed":true}]}` — `skill` = per-app refresh outcomes (one per registered app): `[{"refreshed":true,"app":"opencode",…},{"refreshed":true,"app":"claude-code",…}]` · nothing registered ⇒ `{refreshed:false,reason:"not_registered",warning:"…"}` · `--no-skill` ⇒ `skill:null` |
| `uninstall [--bin-dir <p>] [--purge-config]` | — | `{"uninstalled":true,"bin":"~/.local/bin/carpenter","skill":[{"app":"opencode","path":"~/.config/opencode/skills/carpenter/SKILL.md","removed":true},{"app":"claude-code","path":"~/.claude/skills/carpenter/SKILL.md","removed":true}],"config_purged":false}` — `skill` = per-app removal outcomes (one per registered app): `[{"removed":true,"app":"opencode",…},{"removed":true,"app":"claude-code",…}]` · nothing registered ⇒ `{removed:false,reason:"not_registered"}`; `bin:null` when no binary was present; `NotFound` when neither skill nor binary exists |
<!-- END GENERATED -->

`upgrade` mode resolves: `--source <p>` → config `source_dir` → **GitHub `edge`
release** (curl + `SHA256SUMS` verify + extract — the same pipeline
`scripts/install.sh` uses). Both modes refresh the skill of every **registered**
app (skill file present; opencode `~/.config/opencode/skills/`, claude-code
`~/.claude/skills/`) — nothing is registered that wasn't already (installer
parity with the confirming installer). When no app is registered the outcome is
`{refreshed:false,reason:"not_registered",warning:"…"}`; otherwise `skill` is an
array of per-app outcomes. `--no-skill` skips both (`skill:null`).
The `not_registered` warning string is single-sourced here; see
[adr/004](../adr/004-build-install-split.md), [adr/018](../adr/018-upgrade-fetches-release.md).

`uninstall` is the inverse of `install`/`register` ([adr/019](../adr/019-uninstall-semantics.md)):
skill removal (every registered app, each best-effort —
`{removed:false,reason:"not_registered"}` when none, never fails the run), then
the binary is deleted (safe while running on Linux/macOS). `NotFound` when
neither skill nor binary exists. Config is kept unless `--purge-config`; course
data is never touched.
