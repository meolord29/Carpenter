# ADR-002: Database is the source of truth; notebooks are rendered

Date: 2026-08-08 · Status: Accepted

## Context
Lessons have rich structure (sections, practice questions, quizzes, test cases) with
relations, and this structure plus progress must be tracked. The notebook (`.ipynb`)
is the artifact the learner opens, containing sections → practice → quiz. The two
options were: (A) notebook is the source of truth and the DB mirrors it by parsing
the notebook, or (B) the DB is the source of truth and the notebook is rendered from
it.

## Decision
**The DB is the source of truth; notebooks are rendered from it.** We never parse a
notebook back into structured rows. Rendering is **idempotent**: managed cells
(teaching markdown, check cells) refresh freely; practice/quiz stubs preserve learner
edits via a `scaffold_hash` in cell metadata; any untagged learner cell is always
kept.

## Consequences
+ No fragile notebook parsing — adding structure never depends on regex over cells.
+ Clean ownership contract: the agent owns the DB; the learner owns filled stubs.
+ `lesson sync` is a pure function of (DB, current notebook) → new notebook.
− Rendering must be idempotent and conflict-aware; the `scaffold_hash` rule is the
  safety-critical path (tested explicitly).
− If the DB is lost, the rendered notebooks retain learner code but not the
  structured metadata/cases — so the DB must be backed up / version-controlled.
