# ADR-023: Ruleset branch protection with explicit bypass actors

Date: 2026-08-30 · Status: Accepted

## Context

adr/021 documented branch protection on both trunks; reality was softer.
`main` carried *two* drifting layers — classic branch protection (review
count 0, non-strict checks) and an active ruleset (count 1, strict) — and
neither listed the `guard` job as required, so "only `nightly` merges into
`main`" went red but never blocked a merge. `nightly` had no protection at
all (adr/022's constraint 3 leaned on that: the owner direct-pushes
generated-surface fixes). Meanwhile three automation actors *must* keep
pushing branches directly:

| actor | identity | pushes |
|---|---|---|
| patch ladder + recut | `github-actions[bot]` via `GITHUB_TOKEN` | `chore(release): bump` commits onto `nightly`; post-promotion fast-forward |
| promotion bump | release-bot GitHub App (4769844) | main's minor+1.0.0 onto `nightly` at PR open |
| owner | meolord29 (53997283) | merges; direct-pushes generated-surface drift fixes |

A `pull_request` rule blocks every direct push unless the pusher is a bypass
actor — protection that forgets the bots breaks the adr/022 ladder on the
first merge (bump's push would be rejected).

## Decision

Rulesets only, one per trunk, API-managed:

- **`trunk protection (main)`** (id 20971401, `~DEFAULT_BRANCH`): PR rule
  (1 code-owner approval), 8 required checks — the seven lane contexts plus
  `guard (only nightly merges into main)`, making the channel rule
  merge-blocking — strict up-to-date, no force-pushes, no deletions. Owner
  bypass: `pull_request` (merges via PR only).
- **`trunk protection (nightly)`** (id 21858277, `refs/heads/nightly`):
  PR rule (1 code-owner approval — with `CODEOWNERS * @meolord29`, every
  merge is owner-approved), the 7 lane checks (strict), no force-pushes, no
  deletions.
- **Bypass actors on `nightly`**: owner `always`; release-bot App
  (`Integration` 4769844) `always`; `github-actions[bot]` **as a User-typed
  actor** (41898282) `always` — the API rejects `Integration` 15368
  ("GitHub Actions") on user-owned repos ("must be part of the ruleset
  source or owner organization"), but accepts the bot's user id, and
  `GITHUB_TOKEN` pushes are attributed to that account.
- Classic branch protection on `main` deleted; the ruleset already carried
  the stricter settings. One source of truth per trunk.

## Consequences

+ `nightly` merges are owner-only in mechanism, not convention: non-bypass
  actors can open PRs but cannot merge, push, force-push, or delete either
  trunk.
+ The ladder is untouched — `GITHUB_TOKEN` semantics (no workflow cascade)
  preserved; bump/recut/promote-bump push under bypass.
+ `recut`'s "recreates nightly if it was deleted" branch is now unreachable
  (deletion rule) — kept as harmless dead text.
− Bypass is per-actor all-or-nothing: the owner *can* merge with pending
  checks or push unverified commits; "green before merge" stays policy
  (visible in CI), not mechanism.
− Bot-as-User bypass actors don't render in the ruleset UI picker —
  **re-saving ruleset 21858277 through the UI would silently drop
  `github-actions[bot]`**. Manage it via API.
− The bots' bypass means any workflow in the repo can push `nightly`
  (same-repo PR branches run workflows with `contents: write`); accepted in
  a solo repo, revisit before adding collaborators.

## Rejected

- **`Integration` 15368 as bypass** — the API rejects it for user-owned
  repos.
- **App token for bump/recut instead of a bot bypass** — breaks the
  deliberate no-cascade `GITHUB_TOKEN` design (adr/022 constraint 2): every
  bump would fire a redundant release run; recut would roll the channel
  early.
- **No PR rule on `nightly`** (direct pushes allowed) — guts owner-only
  merges, the point of the exercise.
- **Keeping classic protection on `main`** — two drifting layers; rulesets
  are the maintained path.

Extends [adr/021](021-nightly-main-channels.md) and
[adr/022](022-automated-version-ladder.md).
