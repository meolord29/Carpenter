<!-- Ground rules for merging into `nightly` (adr/021). CODEOWNERS is
     `* @meolord29` — the repo owner reviews every PR and verifies each box. -->

## What this PR does

<!-- Explain what the feature/change does and why a learner or agent would
     care. One short paragraph beats a changelog dump. -->

## Ground rules (feature PRs into nightly)

- [ ] All unit tests pass (`cargo test --workspace` green in CI).
- [ ] New/changed behavior ships with unit tests (a new command fn lands its
      `#[test]` by construction; other logic gets a case added).
- [ ] If the CLI surface or study workflow changed:
      `.opencode/agents/carpenter-dev-validate.md` (the QA agent's
      checklist/prompt) is updated to match in this PR.
- [ ] A **carpenter-dev-validate report** is attached below — **required when
      this PR changes learner-facing or course-building surface** (commands,
      envelope shapes, study workflow, howto contract): the subject-learning
      simulation ran smoothly end to end, verifying existing features **and**
      the new/changed ones. Infra/docs/release-process-only PRs: tick with a
      one-line `N/A — no course-surface change` plus the targeted contract
      validation that ran instead.

### dev-validate report

<!-- Required for course-surface PRs: paste the report (or its summary) —
      commands audited, failure tally (must show zero bugs), and the
      confirmation that simulating learning the subject succeeded smoothly.
      Run the agent with the PR branch checked out. Otherwise: one line
      explaining the N/A + what contract validation covered the change. -->

---

## Promotion PR (nightly → main only)

- [ ] `version` bumped in `Cargo.toml` (and `Cargo.lock`) in this PR.
- [ ] Head branch is `nightly` — the `guard` check enforces this.
