# ADR-019: `uninstall` semantics

Date: 2026-08-18 · Status: Accepted

## Context

`install`/`upgrade`/`register` had no inverse: removing carpenter meant manually
deleting `~/.local/bin/carpenter`, the opencode `SKILL.md` + permission key, and
(optionally) `~/.config/carpenter/config.json`. Supported platforms are Linux and
macOS only — Windows is explicitly out of scope (design/17), which makes
self-deletion tractable.

## Decision

`carpenter uninstall [--bin-dir <p>] [--purge-config]`:

- **Resolution:** `--bin-dir` → config `bin_dir` → default — identical to
  `install` (one resolution path, no drift).
- **Ordering:** skill first, binary last. The skill is recoverable via
  `register`; the binary delete is the point of no return, so it goes last and
  its failures abort before anything else is reported done.
- **Skill removal is best-effort** (mirrors `upgrade`'s `skill_outcome_for`):
  outcome JSON `{removed:true,app,path}` or `{removed:false,
  reason:"not_registered"}`; it never fails the run.
- **Self-delete:** plain `remove_file` on the running binary — safe on
  Linux/macOS, where unlink keeps the inode alive for the running process (the
  same property `upgrade`'s copy-replace and `strip_deleted` rely on). No
  rename dance needed because Windows (locked images) is unsupported.
- **Nothing found** (no skill file and no binary) → `NotFound` error envelope —
  mirrors `deregister`'s contract on an absent skill.
- **Config is kept by default;** `--purge-config` removes it (the apt
  `remove`/`purge` split — package managers that delete user config by default
  are the outlier). Course data (`<root>/courses`) is **never** touched: it is
  user-owned learning material, not installation state.

## Consequences

+ One command reverses the full `install.sh`/`register` footprint; the envelope
  reports exactly what was (`bin:null` when no binary existed) and wasn't
  (`config_purged:false`) removed.
+ `Data::Uninstall` joins spec 18's generated table via the `models::build`
  example rows (adr/008); envelope-smoke coverage comes with them.

− A re-install after `uninstall` without `--purge-config` finds stale config
  (intended — settings survive reinstall).
