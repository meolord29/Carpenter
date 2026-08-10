# Architecture

```
build.rs          syn-scan commands/ -> fail build if a command fn lacks a /// example
                  block or a paired #[test] fn <name>_* (adr/007).
app.rs            clap wiring + _emit envelope harness. No logic.
manual.rs         howto text = include_str!("howto.gen.md") (xtask-produced).
core/
  store.rs        root resolution; slugify; path helpers; course open/init
  db.rs           rusqlite repo: typed accessors + relation joins (sole SQL surface)
  compare.rs      compare(actual, expected, mode) — single source of compare
                  semantics (parity-tested against the helper's Python impl)
  notebook.rs     render lesson.ipynb from DB; idempotent sync; managed cells
                  (section/practice/quiz/check/skip-config)
  helper.rs       render verification-only helper.py (reads+writes course.db: sets
                  pass_or_fail/last_check on each check — adr/010)
  skill.rs        render SKILL.md from typed fields (NAME/DESCRIPTION/WHAT_THIS_IS/
                  WORKFLOW/PEDAGOGY) — no template (adr/009)
  output.rs       envelope structs -> JSON
  error.rs        CarpenterError hierarchy + codes
models/           serde structs: Data (command success-payload enum, one variant
                  per command — serialized as the envelope `data`), Course/Spec,
                  Lesson/Spec, Section, Checkable, TestCase, Note, Plan, Goal,
                  enums — each *Spec/Data type carries a co-located
                  `mod examples` feeding gen-specs (adr/008)
commands/         mod.rs + course, lesson, plan, goal, quiz, progress, notes,
                  bug, feature, config, venv, skip, link, build, install, upgrade,
                  register, deregister, howto — holds ONLY command fns (helpers in core/)
xtask/            gen-howto: introspect clap Command -> howto.gen.md
                  gen-specs: *Spec/Data types -> docs/specs/*.md
                  build: gen-howto + gen-specs + cargo build (--release for `upgrade`)
```

Rules: `app.rs` is wiring only. Commands return `Result<Data, CarpenterError>`;
`_emit` wraps into one envelope (spec parsing is centralized so a bad spec maps to
`ValidationError`, not a crash). Commands never write SQL except through
`core/db.rs`. One compare module (mirrored by the generated `helper.py`); semantics
locked by [specs/20-helper-contract.md](../specs/20-helper-contract.md).
`register` and `upgrade` share a `write_skill()` helper (renders `SKILL.md` + merges
the permission entry) — `upgrade` calls it conditionally to refresh a registered
skill. `#![deny(missing_docs)]` at the crate root forces a `///` on every public
item (clap reads it as `--help`); `build.rs` fails the build if a command fn lacks
an example block or a paired `#[test] fn <name>_*` (see [adr/007](../adr/007-compile-enforced-command-docs.md)).
`core/skill.rs` is the single source of the `SKILL.md` body — this doc references
it, never duplicates (see [adr/009](../adr/009-skill-assembled-from-fields.md)).
