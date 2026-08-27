# ADR-020: branch-governed release channels

Date: 2026-08-27 · Status: Accepted

## Context

ADR-018 made `edge` the single release channel: every push to `main` rolled it,
and it was simultaneously the default install target and the newest build. That
conflates "untested-by-users" with "what everyone gets" — there was no separation
between stable and unstable, no soak window, and no rollback point (the `edge`
tag was deleted and recreated each push).

Deploy-many-times-a-day safety for a distributed CLI (no server, no user
routing) maps to: a canary channel users opt into, immutable stable releases to
roll back to, and unchanged fast gates on every PR (the existing
`ci.yml` matrix + sub-10-min suite already provide the fast feedback; the `dev`
feature flag (adr/016) already lets incomplete work merge dark).

## Decision

Two long-lived **channel branches** govern publishing (`release.yml`):

```
ivan/<topic> ──PR──▶ main          trunk; gates + PR smoke; publishes nothing
main ──merge──▶ pre-release        rolls the `edge` prerelease (canary channel)
pre-release ──merge──▶ release     publishes stable vX.Y.Z (marked Latest)
```

- Merges to `main` **publish nothing** — trunk stays always-green via `ci.yml`
  and the pre-merge smoke lanes (PR-built artifacts over `file://`), but no
  artifact ships until a channel merge cuts it.
- Push to `pre-release` re-creates the rolling **`edge`** prerelease (canary).
- Push to `release` publishes an immutable **stable `vX.Y.Z`** — the version is
  read from `Cargo.toml` (bumped in the promotion PR) and the release is not
  marked prerelease, so GitHub's **Latest** pointer follows it.
- Each release attaches a **channel-correct installer**: stable's `install.sh`
  is tag-patched at publish (`TAG="vX.Y.Z"`), so the stable one-liner can never
  fetch edge bits; edge ships the stock script. `releases/latest/download/…`
  always serves the newest stable.
- `carpenter upgrade` gains `--channel stable|edge` (default **stable** →
  `releases/latest/download`; `edge` → `releases/download/edge`)
  (`core/release.rs::Channel`). `CARPENTER_DOWNLOAD_BASE` still overrides
  (tests/mirrors).

## Consequences

+ Clear stable/unstable separation: canary users soak `edge`; stable only moves
  by explicit promotion.
+ Rollback = install a previous immutable `vX.Y.Z` tag (documented in DEV.md);
  no delete-and-recreate data loss.
+ Publish cadence decoupled from merge cadence — merges can flow all day
  without shipping anything.
− Two long-lived branches to protect and keep current (the one sanctioned
  exception to trunk-based short-lived branches; promotion PRs are `main →
  pre-release → release` merges plus a version bump).
− A forgotten promotion leaves stable stale; the promotion checklist in DEV.md
  is the manual guardrail.

## Rejected

- **Timed/`workflow_dispatch` nightly cuts** — cadence without intent; a
  promotion merge encodes "this is ready" better than a clock or a button.
- **Canary percentages / automatic rollback on error metrics** — impossible for
  a `curl | sh` CLI with no telemetry; the channel model is the CLI-shaped
  equivalent. Adding telemetry would be its own ADR.
- **Renaming `edge`** — with branches + Latest carrying the stable signal,
  `edge` is the conventional prerelease name; a rename would ripple through a
  dozen pinned surfaces for no clarity gain.
