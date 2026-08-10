# Quiz run (`nbconvert` execution model)

`quiz run` does **not** run a custom subprocess child. It executes the lesson
notebook in the course venv and lets the in-notebook helper cells do the scoring.
nbconvert's kernel is the isolation boundary — the same trusted boundary
[`lesson execute`](16-execution.md) already uses.

## Flow
1. Resolve the lesson's `<NN-slug>/lesson.ipynb`. `StoreError` if the course venv
   does not exist (`carpenter venv create` first).
2. Execute in the venv with errors tolerated:
   ```
   uv run jupyter nbconvert --execute --to notebook --inplace \
       --ExecutePreprocessor.allow_errors=True lesson.ipynb
   ```
3. During execution each `managed=check` cell calls `helper.check(...)`, which
   scores per case and **writes `pass_or_fail` + `last_check`** to the practice /
   quiz row ([06-helper.md](06-helper.md)). Learner errors are caught per case
   (`error:"…"`); an unfilled stub ⇒ all cases fail, no crash.
4. After execution, carpenter inspects the errored cells and **classifies each by
   `scaffold_hash`** (`metadata.scaffold_hash`, see
   [05-notebook-sync.md](05-notebook-sync.md)):
   - hash **unchanged** (cell still equals the scaffold carpenter rendered) on an
     errored managed cell ⇒ **scaffolding bug** — the agent's generated material is
     broken. ⇒ `ExecuteError {details:{errors:[{index,ename,evalue}]}}`. The agent
     must rewrite the section and re-run.
   - hash **changed** (learner edited the stub) ⇒ learner error, already caught and
     scored by the helper as a fail. Not escalated.
5. If no scaffolding cell errored, return `ok`: read `last_check` + `pass_or_fail`
   + `skip` for every quiz under the lesson (skipped quizzes are scored like any
   other but excluded from status derivation).

## Why no custom runner
Earlier drafts proposed a static argv-parameterized Python child. That is
unnecessary: the helper already scores in-notebook, the notebook is the unit the
learner edits, and reusing nbconvert avoids a second execution path and a second
security boundary to defend. Learner code never reaches carpenter's process — it
runs inside nbconvert's kernel.

## What `quiz run` does *not* do
- It writes **no history**. The `attempts` table was removed (adr/010); `last_check`
  is a current-snapshot column, overwritten each run. `quiz results` reads it.
- It does **not** abort on learner errors (an unfilled stub is the normal mid-lesson
  state). Only scaffolding errors escalate.
