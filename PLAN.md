# carpenter — Implementation Plan (living tracker)

Check a box when its **phase-exit gate** is green:
`cargo xtask build` (= gen-howto + gen-specs + build) · `cargo test` (or `cargo nextest run`) · `cargo clippy -- -D warnings` · `cargo fmt -- --check` · `cargo doc --no-deps -D warnings`.

Design source: `docs/design/14-build-order.md`. Module map: `docs/design/03-architecture.md`.
A module is not "done" in its phase until its unit tests pass alongside its mandated command test.

## Principles (locked)

- **Composition over inheritance.** `Data` enum (one variant per command) + plain `serde` structs composed by value. Practice & Quiz share a `Checkable` *shape* but live in separate tables — no trait hierarchy, no shared base.
- **Functional, not OOP.** Core logic = free fns in `core/*.rs` returning `Result<T, CarpenterError>`; models carry no behavior; no `unwrap`/`expect` outside tests.
- **Compile-enforced self-docs.** `#![deny(missing_docs)]` + `build.rs` syn scan: every command fn needs `///` + fenced example + paired `#[test] fn <cmd>_*`.

## Locked decisions

| # | Decision |
|---|---|
| gen-specs | Scaffold generator + `<!-- BEGIN GENERATED -->` markers in P1; tables fill as each `*Spec`/`Data` type lands. |
| Phases | Docs-aligned 0–10 (one-to-one with `design/14-build-order.md`). |
| Status derivation | New `core/status.rs` — pure fns, mirroring `core/compare.rs`. |
| Testing | Unit tests land in the **same phase** as the module they cover. |

## Cross-cutting single-owners

| Concept | Owner | Consumers |
|---|---|---|
| `scaffold_hash` | `core/notebook.rs` | render, sync (3-way), `quiz run` classification |
| compare semantics | `core/compare.rs` (Rust) ↔ generated `helper.py` `_compare` | parity-tested |
| status derivation | `core/status.rs` | `lesson show/list`, `progress`, `goal list`, `quiz results` |
| envelope + `_emit` | `core/output.rs` | every command |
| id allocation (`max+1`) | `core/db.rs` | every insert |
| slug derivation (`kebab`) | `core/store.rs` | course/lesson create |

---

## Phase 0 — Foundation (gates + skeleton)

- [x] Workspace: `Cargo.toml` members `carpenter` (bin) + `xtask` (bin)
- [x] Deps added as used: `clap`(derive), `serde`, `serde_json`, `thiserror`, `dirs` + `syn`(build-dep). `rusqlite`/`validator` deferred to P2/P4.
- [x] `#![deny(missing_docs)]` at crate root
- [x] `build.rs` syn scanner — fails build if a `commands/` fn (`pub fn -> Result`) lacks `///`+example or paired `#[test] fn <cmd>_*`
- [x] `app.rs` clap wiring: global `--version`, `--root`, `--course/-c` + top-level command groups
- [x] `core/error.rs`: `CarpenterError { NotFound, AlreadyExists, ValidationError, StoreError, ExecuteError, Conflict }` via `thiserror`
- [x] `core/output.rs`: `render()` pure envelope → JSON; `app::emit` wraps (ok → exit 0, error → exit 1)
- [x] `core/store.rs`: root resolution, `~/.config/carpenter` helpers, `slugify`
- [x] `manual.rs`: `include_str!("howto.gen.md")` placeholder
- [x] `howto` stub command round-trips through `emit`
- [x] **Unit tests:** store (slugify cases + config_dir); output (ok+error serialization); error (variant→code); app (cli has howto); howto fn — 12 tests pass
- [x] Phase-exit gate green

**Notes (P0):**
- **`Cargo.toml` `[alias]` is invalid** — cargo aliases live in `.cargo/config.toml`. `xtask` alias added there. (Remember for any future aliases.)
- ✅ **DONE** `cargo doc -- -D warnings` errors in this toolchain (`-D` rejected). `AGENTS.md` build block now uses `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`.
- **`build.rs` `peel` had a `never_loop` bug** (only peeled one `Paren` level) — clippy caught it; fixed to peel all levels.
- ✅ **DONE** `slugify` now applies NFC (added `unicode-normalization`) for spec fidelity — no observable effect, but matches `conventions`. "Intentionally omitted" note removed.
- Deps kept lean: `rusqlite`(bundled, compiles SQLite from source) + `validator` not added until first used (P2/P4) to keep cold builds fast.
- Slug collision dedup (`-2`,`-3`): courses return `AlreadyExists` on duplicate slug (no auto-suffix, per `08-course` spec). Lesson slug auto-suffix semantics TBD at P4.
- `emit` is private in `app.rs`; the pure/testable part is `core::output::render` (functional split, no envelope struct leaking into wiring).

## Phase 1 — Codegen pipelines

- [x] `xtask gen-howto`: introspect real `clap::Command` + per-command example scan → `src/howto.gen.md`; runs via `cargo xtask build`
- [x] `howto` command prints `howto.gen.md` (via `manual::MANUAL`)
- [x] `xtask gen-specs` skeleton: reads `models::examples` registry; replaces only between `<!-- BEGIN/END GENERATED -->`; preserves surrounding prose
- [ ] Marker regions added to `docs/specs/*.md` — **only `19-howto.md` is generated now**; other files keep hand tables until their types land (per-file as P2+ ships)
- [x] **Tests:** howto stale-check (regenerate == committed); gen-specs marker freshness; `skill` determinism deferred to P9 (module not yet present)
- [x] Phase-exit gate green (`cargo test --workspace` 13+2; clippy/fmt/doc `--workspace` clean)

**Notes (P1):**
- ✅ **DONE** `cargo test` only covers the root package in this non-virtual workspace; canonical commands now documented with `--workspace`/`--all` in `AGENTS.md` (test, clippy, fmt, doc all use it; doc deny via `RUSTDOCFLAGS`).
- 🐛 **CWD-relative paths broke tests**: gen logic used relative paths (`src/commands`, `docs/specs`); `cargo test` runs xtask with CWD=`xtask/`, so examples weren't found → manual lacked the example block → false stale. Fixed: all xtask paths resolve from the workspace root via `CARGO_MANIFEST_DIR` (`xtask/src/paths.rs`).
- **Example extraction**: `/// foo` doc literals carry a leading space; `doc_text` trims leading whitespace per line. Global flags (`--root`,`--course`) listed once at the top; per-command shows only its own (filtered by the root's long-name set).
- **Source scan duplicated** between `build.rs` (enforcement) and `xtask` (generation) — both parse `commands/` with syn (~15 lines each). Accepted; factor a shared `scan` crate if it grows.
- gen-specs touches only files with a registry entry (`models::examples::all()`); `missing_docs` lands on the one pub `all()` fn, not each example (adr/008). `19-howto.md` now fully generated.
- `cargo xtask build` = gen-howto + gen-specs + `cargo build` subprocess (uses `$CARGO`).
- Verified drift detection fires (tampering `howto.gen.md` → stale-check fails).

## Phase 2 — Storage + Course CRUD

- [x] `core/db.rs`: open `course.db`, `PRAGMA foreign_keys=ON`, single `CREATE`-based schema (`data-model/03-ddl.md`), typed accessors for `course_meta`, id allocation (`max+1`, never reused)
- [x] `models/course.rs`: `CourseSpec`, `CourseRow`, `CourseListItem`, `CourseCounts`
- [x] `commands/course.rs`: `create --spec -`, `list`, `show`, `update --spec - --force`, `delete --force`, `switch`
- [ ] `core/status.rs` — **deferred to P3** (YAGNI: no derivation fn needs it yet)
- [x] **Unit tests:** `db` (id monotonic/never-reuse via `id_seq`, course_meta roundtrip, counts, idempotent open), `time` (epoch + 1700000000), `config` (roundtrip), `store` (atomic_write); 11 course-command tests (create/list/show/update/delete/switch incl. conflict + not-found)
- [x] Phase-exit gate green (`cargo test --workspace` 34+2; clippy/fmt/doc `--workspace` clean; e2e verified via real CLI)

**Notes (P2):**
- ✅ **DONE** `id_seq` counter table documented in `docs/data-model/03-ddl.md` (honors "deleted ids never reused"; `next_id` uses `INSERT … ON CONFLICT … RETURNING`).
- 🔧 **`gen-howto` upgraded to recurse nested groups** (Phase 2 added the `course` group); examples now keyed `<module>::<fn>` to avoid `create`/`list` collisions across groups (course/lesson/…). `src/howto.gen.md` regenerated.
- ⏳ **`core/status.rs` deferred to P3** (YAGNI — created when the first derivation fn lands).
- ✅ **DONE** gen-specs now populates `02-course-spec.md` + `08-course.md` (real serialized `Data`/`CourseSpec` shapes + authored rules/notes; `models/course.rs::examples` co-locates example values). Stale-check guards them. Future types register the same way.
- `core/time.rs`: ISO-8601 UTC with no time crate (Howard Hinnant civil-from-days). `core/config.rs`: path-based `load_from`/`save_to` (testable). Both use atomic writes.
- `Paths { root, config_dir }` context struct → command fns are pure over `&Paths` (no globals/env vars). app.rs builds it from `--root` + `config_dir()`.
- `rusqlite` (bundled) added — cold build compiles SQLite from C source (~33s); incremental builds are fast.
- `course list` checks `course.db` existence before opening (avoids creating a stray db in non-course dirs).

## Phase 3 — Plans + Goals

- [x] `models/`: `PlanSpec`/`Plan`, `GoalSpec`/`Goal` + co-located `mod examples`
- [x] `commands/plan.rs`: `create`/`show`/`list`/`confirm`/`update`/`delete` (human-in-the-loop)
- [x] `commands/goal.rs`: `add`/`list`/`update`/`remove`
- [x] `plan confirm` (course scope): insert `goals` rows + resolve `covered_by` (range-check at create, resolve at confirm)
- [x] `core/status.rs`: goal derivation (`override`→authored; all `covered_by` complete→`achieved`; empty→`pending`) **+ full lesson derivation** (implemented ahead of P5 since goal derivation needs lesson status; it's the single-owner module)
- [x] gen-specs populated for plan (`04`,`10`) + goal (`05`,`11`)
- [x] **Unit tests:** goal status branch (override pin, empty covered_by→pending, derived-clears-override); plan confirm link range-check + resolve; 11 plan + 8 goal command tests; status rules
- [x] Phase-exit gate green (`cargo test --workspace` 56+2; clippy/fmt/doc clean; e2e verified)

**Notes (P3):**
- 🔧 **`core/status.rs` implements BOTH lesson + goal derivation now** (not just goal) — goal derivation needs lesson status, and status.rs is the single owner. Lesson derivation reads `pass_or_fail`/`skip` (all 0 in P3, so lessons → `not_started`); it yields real statuses once lessons + items land (P4–P5). **P5's lesson-derivation task is effectively done** — P5 just consumes it.
- ✅ **DONE (P8)** Lesson-scope plan creation: `plan create --scope lesson --lesson <id>` now works (the lesson must exist; `scope_id` = lesson id). `plan confirm` for a lesson-scope plan creates no goals (course-scope only). See P8 notes.
- Plan/goal commands operate on the **active course** (resolved from `--course` or `config.active_course` via new `app::active_course` + `db::open_course`). No active course ⇒ `ValidationError`.
- `--covered-by` takes a comma-separated id list (single flag). Plan `content` stored as JSON `{goals, links}` and re-parsed on confirm. `goal list` shows effective (override-aware) + derived status; `goal update --status derived` clears override and recomputes.
- Shared helpers added: `store::parse_spec` (course/plan/goal), `commands/testutil` (course-backed test setup), `output_table` in `examples.rs` (3-col output-contract table for any command group).
- `override` column ↔ Rust field `override_flag` via `#[serde(rename = "override")]` on the `GoalUpdate` variant.

## Phase 4 — Lesson authoring

- [x] `models/lesson.rs`: `LessonSpec`, `Section`, `Snippet` (`snippets[0].kind=="markdown"`), `Checkable`, `TestCase` + `mod examples`
- [x] `core/compare.rs`: Rust `compare(actual, expected, mode)` (`exact`/`sorted`/`set`; `unsortable`/`unhashable`)
- [x] `core/notebook.rs`: render DB → `lesson.ipynb` (skip-config → title → section-md/code → practice-stub+check → quiz-stub+check); `metadata.managed` tags; `scaffold_hash` (FNV-1a)
- [x] `core/helper.rs`: render generic `helper.py` (stdlib-only; `check`; single constrained `UPDATE`; `is_skipped`; never prints `expected`)
- [x] `commands/lesson.rs`: `create --spec -` (DB inserts + render notebook + helper), `get <id>` (full tree). (list/show/update/delete/sync/execute land in P5/P6)
- [x] gen-specs populated for `03-lesson-spec` + `09-lesson`
- [x] **Unit tests:** `compare` (9: exact/sorted/set + unsortable/unhashable + 6==6.0); `notebook` (scaffold/hash determinism, check_source); `helper` (surface + never-prints-expected); `lesson` (create renders nb+helper, rejects non-md first snippet, slug dedup, get tree)
- [x] Phase-exit gate green (`cargo test --workspace` 73+2; clippy/fmt/doc clean; e2e verified)

**Notes (P4):**
- ✅ **Lesson slug auto-suffix (`-2`,`-3`,…) implemented** — resolves the P3 "TBD at P4" item. `unique_lesson_slug` loops until free; collision on `arrays-101` → `arrays-101-2`.
- `scaffold_hash` = **FNV-1a 64-bit hex** (stable across versions/platforms; no extra crate). Lives only in cell metadata. Canonical scaffold = `signature` + `# prompt` + `raise NotImplementedError` (4-space indent).
- **`helper.py` is a generic const** — resolves `course.db` via `__file__` (`parents[2]`); scores via `_compare` (mirrors `core/compare`); writes `pass_or_fail`/`last_check`; **never prints `expected`**. Parity test (Rust compare ↔ Python `_compare`) lands in P6 when nbconvert runs it.
- **Render order:** skip-config → title → **per-section** (snippets then practice stubs+checks) → quizzes+checks. Practice is attached to its teaching section (domain-model-faithful); quizzes at the notebook end.
- **skip-config cell** delegates to `helper.is_skipped` (notebook cells have no `__file__`, so the DB-path resolution lives in the helper).
- snippet ids: global per-lesson counter `sn1`…, stored in `sections.snippets` JSON (stable across syncs — needed by the P5 3-way match).
- `CompareMode` derives `Serialize` too (it's a field of the gen-specs `LessonSpec` example).
- gen-specs `09-lesson.md` shows `create`+`get` rows now; grows in P5/P6 as more lesson `Data` variants land.

## Phase 5 — Lesson lifecycle

- [x] `lesson sync [--force]`: 3-way via `scaffold_hash` (untouched→refresh; learner-edited & DB same→keep; learner-edited & DB changed→conflict, `--force` overwrites); `conflicts[]` (`db_changed`); managed non-stub cells regenerated wholesale; untagged learner cells preserved (re-anchored)
- [x] `lesson list`/`show`/`update`/`delete` (`--force` on update/delete)
- [x] `core/status.rs` lesson derivation — **already done in P3** (consumed here)
- [x] gen-specs `09-lesson.md` rows extended to all 7 lesson commands
- [x] **Unit + integration tests:** sync preserves a learner-edited stub (db unchanged); sync reports `db_changed` conflict when DB signature changes under a learner edit (and leaves it intact w/o `--force`); list/show/update(+conflict)/delete(+conflict)
- [x] Phase-exit gate green (`cargo test --workspace` 81+2; clippy/fmt/doc clean; e2e verified)

**Notes (P5):**
- **3-way sync** (`core/notebook.rs::sync_notebook`): `learner_touched = hash(cell_source) != cell.metadata.scaffold_hash`; `db_changed = canonical_now_hash != scaffold_hash`. Conflict only when both — reason `"db_changed"`. **Simplification:** a learner edit with NO DB change is silently preserved (not reported); the spec's `"learner_edited"` reason is reserved/unused for now. ⚠️ Note this reading of the (slightly internally-inconsistent) sync design.
- **Learner-cell preservation:** untagged cells are re-anchored after their preceding managed cell (keyed by `managed_key`); orphans (anchored to a removed managed cell) are preserved at the notebook end.
- Managed **non-stub cells** (title/skip-config/section/check) are regenerated wholesale on sync (outputs stripped — the fresh render already has empty outputs).
- `update` overwrites the notebook with a fresh render (resets stub learner edits); it does **not** 3-way preserve. Acceptable — `update --force` is an explicit content change before learners typically start. ⚠️ A sync-preserving `update` is a possible refinement.
- `delete_lesson_content` manually deletes a lesson's `test_cases` (no FK on `test_cases.owner_id`) before cascading sections/quizzes.
- `LessonDb` gained `created_at`/`updated_at` (needed for `update`'s `updated:` row).

## Phase 6 — Execution (venv + nbconvert)

- [x] `commands/venv.rs`: `create` (`uv venv` + `pyproject.toml` base deps + `uv sync`), `sync`, `list`, `add`; `StoreError` if no `uv`; `AlreadyExists` if `.venv` present
- [x] `lesson execute`: `uv run jupyter nbconvert --execute --inplace`; strict (default → `ExecuteError{index,ename,evalue}`) or `--allow-errors` (→ `{cells,errors[]}`); `StoreError` if no `.venv`
- [x] `quiz run`: nbconvert `--allow_errors`; helper cells write `pass_or_fail`/`last_check`; classify errored stub cells via `scaffold_hash` (unchanged→`ExecuteError`; learner-edited→scored as fail); returns per-quiz `{skipped,pass_or_fail,passed,total,cases[]}`
- [x] `quiz list`/`show`/`results`
- [x] **Integration tests:** real `venv create` + `quiz run` (nbconvert) verified via CLI — helper scored `2/2`, `lesson show` rolled up to `complete`. Rust↔Python compare parity (runs generated `helper.py` `_compare` via `python3`); `helper.py` syntax-checked by a test
- [x] gen-specs for `12-quiz` + `22-venv`
- [x] Phase-exit gate green (`cargo test --workspace` 95+2-ignored+2; clippy/fmt/doc clean)

**Notes (P6):**
- 🐛 **`helper.py` docstring bug found via e2e:** `r#"""carpenter` opened a raw string whose `r#"` delimiter consumed one quote, so the file started with `""carpenter` (a broken docstring) → `import helper` failed everywhere. Fixed by switching the header to `#` comments. Added `helper_is_valid_python` test (ast.parse via `python3`) which would have caught it. ⚠️ **Lesson:** raw strings starting the content with `"""` need `r##"` or a non-docstring opener.
- 🔧 **nbconvert runs from the lesson dir** (cwd = `lessons/<NN>/`), not the course dir — so the kernel cwd resolves `import helper` (helper.py lives next to the notebook). `uv run` walks up to find `pyproject.toml`/`.venv`.
- **`uv` presence** is checked up front (`exec::uv_available`/`require_uv`); a missing `uv` → a clear `StoreError` naming uv (per the user's instruction). `uv` 0.11.3 confirmed on this machine.
- nbconvert is always invoked with `--ExecutePreprocessor.allow_errors=True` (so all cell errors are captured in outputs); "strict" `lesson execute` then reports the *first* error as `ExecuteError` (a deliberate simplification — later cells still run; robust + parseable). ⚠️ Deviates from "strict aborts on first" in process, not in result.
- `ExecuteError` upgraded from `ExecuteError(String)` to `ExecuteError { message, details }` (carries `{index,ename,evalue}` or `{errors:[…]}` in the envelope `details`).
- nbconvert integration tests are `#[ignore]` (need a real venv + jupyter download — run manually); the **parity** + **helper-syntax** + **scan-errors** tests run in the normal gate and are fast.
- `CheckableDb` gained `last_check`; added `db::quiz_lesson_id` + `owner_case_count`.

## Phase 7 — skip

- [x] `commands/skip.rs`: top-level `--scope lesson|quiz|practice <id> [--off]` (sets `skip` column; adr/011)
- [x] Status derivation excludes skipped items; `lessons.skip=1`→`skipped` — **already implemented (P3 `lesson_status_inputs` filters `skip=0`; P3 `derive_lesson`)**; P7 added the tests proving it end-to-end
- [x] Rendered into `managed=skip-config` cell on next sync (not on execute) — **satisfied by construction since P4**: the cell delegates to `helper.is_skipped`, which reads the columns live from `course.db`; managed ⇒ regenerated wholesale on sync
- [x] **Unit tests:** `skip` sets column per scope; derivation excludes skipped; `NotFound` on bad id; `skip_*` command fn (6 new: 5 command + 1 db)
- [x] Phase-exit gate green (`cargo test --workspace` 101+2-ignored+2; clippy/fmt/doc clean; e2e verified via real CLI)

**Notes (P7):**
- **`--scope` is required** (no default, no id-prefix inference — lesson ids are slugs with no prefix; adr/011/spec-23 always show it). cli-surface.md's brackets denote a flag, not optionality.
- 🔧 New `core/db.rs` accessors: `get_practice` (NotFound-aware), `practice_lesson_id` (practice→section→lesson), `set_skip(table,id,val)` (table is an internal constant), `set_lesson_status` (refreshes the denormalized `lessons.status` cache per data-model/04 — read paths still recompute, so the cache is belt-and-braces for P8's `progress`).
- `Data::Skip {scope,id,skip}` + `models/skip.rs::examples`; gen-specs now populates `23-skip.md` (markers added around the table; prose untouched). Stale-check guards it.
- Derivation-exclusion test sets `pass_or_fail=1` via raw conn (the helper's write path is Python-only), then skips the quiz ⇒ lesson flips `in_progress`→`complete`→back on `--off`.
- 🧹 ✅ **DONE (P8)** Drive-by: `docs/specs/README.md` no longer claims only `19-howto.md` carries markers — rewritten to the registry-driven rule.

## Phase 8 — Progress + Notes

- [x] `commands/progress.rs`: `show` (per-lesson `{status,skip,passing,total}`), `summary` (lessons/quizzes/goals/notes roll-up incl. `notes.by_kind`)
- [x] `commands/notes.rs`: `add` (advisory `related_open` = open notes sharing ≥1 tag, excludes self), `show`/`list` (`errors[]`), `update`/`resolve`/`remove`; `recurrence` author-owned
- [x] **Unit tests:** `related_open` tag-overlap (excludes self); corrupt-row surfacing in `errors[]`; `progress.summary` roll-up math; note/progress command fns
- [x] Phase-exit gate green (`cargo test --workspace` 120+2-ignored+2; clippy/fmt/doc clean; e2e verified via real CLI)

**Notes (P8):**
- `notes` is course-scoped (active course, like goal/lesson). `NoteSpec` is parsed as plain `String`s for `kind`/`recurrence` and validated inside the command fn (clear message; matches `goal update`'s status style) rather than via a serde enum. `tags` default `[]`, `recurrence` default `new`, `related` default `""` (free text, no FK — unresolvable ids kept as-is per spec).
- **`related_open`** = open notes sharing ≥1 tag with the new note, excluding itself; computed in Rust over `list_notes` (skips rows whose `tags` JSON won't parse). Advisory only — it is echoed but `recurrence` is **never** auto-flipped (spec 14). Resolved notes are excluded even when they share a tag.
- **Corrupt rows:** only `tags` (JSON) can be corrupt (other columns are CHECK-constrained in SQL). `notes list` surfaces a bad-tags row as `errors[]:{id,reason:"corrupt tags json for <id>: …"}`; `notes show` (one id) raises `StoreError`. The helper never writes bad JSON — a corrupt row is only reachable via direct DB tampering (the test injects one via raw SQL).
- **`progress show`** reads `lesson_status_inputs` for both the derived status and `passing`/`total` (non-skipped practice+quiz). **`progress summary`** rolls up: lessons by derived status (each lesson re-derived), quizzes `(passing,total)` over **non-skipped** quizzes (one `SUM`/`COUNT` query), goals `(total,achieved)` override-aware via `status::goal_effective`, notes via one 8-column aggregate query (`note_counts`). No `--course` filter needed — one `course.db` per course.
- `notes.by_kind` is an object keyed by the 5 kinds (`gap`/`mistake`/`strength`/`pattern`/`progress`), field order matching the spec.
- gen-specs now populates `06-note-spec` + `13-progress` + `14-notes` (markers added; tables regenerated from `models::{note,progress}::examples`). Stale-check guards them.
- ✅ **DONE (P7 drive-by):** `docs/specs/README.md` no longer claims only `19-howto.md` carries markers — rewritten to state the rule (registry-driven) + list the two pure-prose files, so it won't go stale as more types land.
- ✅ **DONE (P3 ⚠️ "revisit at P4"):** lesson-scoped `plan create` landed. `--scope lesson` now requires `--lesson <id>` (a lesson that must exist — else `NotFound`); `scope_id` is that lesson id. `--scope course` rejects `--lesson` (explicit over implicit). `plan confirm` for a lesson-scoped plan creates **no** goals (the `goals` table is `CHECK (scope IN ('course'))` — course-scoped only). The old "only --scope course supported" rejection + its test are replaced by 4 lesson-scope tests + a `confirm_lesson_scope_creates_no_goals` test.
- ⏳ **P5 "sync-preserving `update`" refinement** considered, kept deferred (YAGNI — `update --force` is an explicit authoring step before learners typically start).

## Phase 9 — Meta commands

- [x] `commands/bug.rs` + `commands/feature.rs`: file-backed JSON under `~/.config/carpenter/{bug,feature_request}/`; `file`/`list`/`show`/`resolve`
- [x] `commands/config.rs`: `get`/`set` (typed coercion; unknown key → `ValidationError`)
- [x] `commands/link.rs`: manifest (`register`)
- [x] `commands/build.rs`, `commands/install.rs`, `commands/upgrade.rs`
- [x] `core/skill.rs`: `render()` (frontmatter + sections from typed fields; adr/009)
- [x] `commands/register.rs`/`commands/deregister.rs`: write/remove `~/.config/opencode/skills/carpenter/SKILL.md` + merge `"skill":{"carpenter":"allow"}` into `opencode.json`
- [x] **Unit tests:** bug/feature `max+1` per kind + `repro`/`rationale` mutual exclusion; `config` coercion + unknown-key; register/deregister JSON merge (touches only `carpenter` key, preserves the rest); `upgrade` skill branches (`refreshed`/`not_registered`/`--no-skill`→null) + source-resolution error paths; skill determinism + frontmatter regex; meta command fns
- [x] Phase-exit gate green (`cargo test --workspace` 164+2-ignored+2; clippy/fmt/doc `--workspace` clean; e2e verified incl. a real `upgrade` release build)

**Notes (P9):**
- **`core/bugfile.rs`** owns file-backed CRUD (`bug/`, `feature_request/`; ids `b1`/`f1`… = `max+1` per kind over existing files; never reused since there is no `delete`). `commands/bug.rs` + `feature.rs` are thin wrappers over a shared `Kind`. **Shared `Data::Issue{File,List,Show,Resolve}` variants** — bug/feature have identical shapes (spec 15 is one table); `show` carries both `repro?`/`rationale?` (the union matches spec 15 exactly). **`--spec` only** — the `--title/--description` flag form was dropped from spec 07/15 for consistency with every other command. `list` surfaces corrupt files in `errors[]` (consistent with course/lesson/notes).
- **`core/skill.rs`** renders `SKILL.md` from typed consts (`NAME`/`DESCRIPTION`/`WHAT_THIS_IS`/`WORKFLOW`/`PEDAGOGY` + version + bin via `current_exe`) — no template (adr/009). **Skill-determinism test** (re-render byte-equal) + frontmatter `name` regex `^[a-z0-9]+(-[a-z0-9]+)*$` — the **P1-deferred gate lands here**. **No `gen-skill` xtask** (design/03 line updated; `register --print-skill` + the determinism test cover it — adr/009 says no committed artifact is diffed).
- **opencode skills dir is a SIBLING of `carpenter/`** under the XDG root (not nested in `~/.config/carpenter/`). New `Paths::xdg_root()` = `config_dir.parent()` resolves it; `testutil::meta_setup` now nests `config_dir` under a unique parent so `xdg_root()` is per-test-isolated. Permission merge via `set_nested` (creates `permission.skill.carpenter` on demand; preserves all other keys — merge never overwrite). `--app` defaults `opencode`; `claude-code`/`agents` ⇒ `ValidationError`; **no TTY prompt** (envelope policy forbids it).
- **build/install/upgrade:** `store::init_course_dir` is shared by `course create` + `build` (the on-disk course shape can't drift between them — `course create` refactored to call it). `build <path>` scaffolds at an arbitrary path (slug from basename + `lessons/`). `install` copies `current_exe` → `bin_dir` (default `config.bin_dir`). `upgrade` resolves `--source` → `config.source_dir` → `ValidationError`; runs `cargo xtask build --release` (**xtask `build` gained a `--release` passthrough** per adr/004, so the embedded `howto` regenerates); parses the new version from `<bin> --version`; atomic tmp+rename replace; best-effort skill refresh (`refreshed` / `not_registered` single-sourced warning / `--no-skill`⇒`null`). `link register` emits a compute-only manifest (`commands` from `app::cli()`).
- **`source_dir` added to config** (ADR-004 needed it for `upgrade`; spec 16 + `config.rs` updated). `config get` (all) returns the effective config with defaults applied; optionals `null` when unset.
- 🐛 **Drive-by test-isolation fix:** the `AtomicUsize::new(0)`-per-call pattern in several test `cfg_dir`/`tmp` helpers always returned `0`, so parallel tests collided on the same temp dir (surfaced as a flaky skill test). Fixed to proper `static` counters in `core/{skill,bugfile,config}` tests.
- ✅ **Resolved P5 sync drift:** `design/05` + `spec/01` tightened — `lesson sync` `reason` is currently always `db_changed` (a learner edit with no DB change is silently preserved, not reported; `learner_edited` is reserved).
- ✅ **P1 marker-regions item fully resolved:** every spec with a `*Spec`/`Data` pair now carries `<!-- BEGIN/END GENERATED -->` markers (only pure-prose `01-envelope`, `20-helper-contract` never do, per `specs/README.md`).
- ⏳ **P5 "sync-preserving `update`" remains deferred** (YAGNI — `update --force` is an explicit authoring step before learners start; reaffirmed P5+P8).

## Phase 10 — Hardening & lock-down

- [x] Parametrized **envelope smoke test** across all commands (doubles as spec goldens) — `envelope_smoke_round_trips_every_example` + error-variant sweep (`models/examples.rs`); one source shared with gen-specs (Option A)
- [x] Drift suite: howto stale-check, spec-marker regen, skill determinism, compare parity, sync-preservation golden — all in `cargo test --workspace` (regenerate-to-buffer + byte-equality; **hosted CI deferred — repo has no host yet**)
- [x] `AGENTS.md` reviewed/current (false CI sentence fixed; `Data`→`rows()` guidance added)
- [x] Full gate green end-to-end (`cargo test --workspace` 166+2-ignored+2; clippy/fmt/doc `--workspace` clean)

**Notes (P10):**
- **Envelope smoke test** = `envelope_smoke_round_trips_every_example` (`models/examples.rs`): iterates `envelope_examples()` — one pub fn flattening every group's `rows()` + `howto_data()` (the **same** source `output_table`/gen-specs consume, adr/008) — through `core::output::render`; asserts `status:"ok"` + non-empty `message` + the envelope `data` == `serde_json::to_value(&example)` (golden parity from one source — no third representation, DRY). Plus `envelope_renders_every_error_variant` sweeping all 6 `CarpenterError` → `status:"error"` + `code`. Chose **Option A** over full e2e goldens (Option B): B would add ~40 committed golden files + a normalizer + fixture builder and a *third* pinning of the contracts (spec tables + per-command tests + goldens) — tensions the locked DRY/YAGNI principles; A reuses the single `models/examples` source.
- 🐛 **Spec-coverage gap found & fixed:** `Data::LessonExecute` had no `rows()` entry, so `09-lesson.md` was missing the `execute` row (undetected since P6). Added the example (mirrors the fn's own `///` example exactly); `gen-specs` regenerated `09-lesson.md`. AGENTS.md "When you change something" now states every `Data` variant needs a `rows()` example or the spec table + smoke test miss it.
- **Drift suite — no hosted CI (per decision):** the repo isn't a git repo and has no host, so the prior "CI runs `xtask build` then `git diff --exit-code`" claim was false. All 5 drift checks already run in `cargo test --workspace` and regenerate-to-buffer + assert byte-equality with committed files (no git/CI needed): howto stale (`howto_gen_md_is_fresh`), spec-marker (`specs_marker_regions_are_fresh`), skill determinism (`render_is_deterministic`), compare parity (`parity_with_helper_python`, skips w/o `python3`), sync goldens (`sync_preserves_learner_edited_stub` + `sync_conflicts_when_db_changed_under_learner_edit`). AGENTS.md reworded to describe this real enforcement.
- **AGENTS.md review:** fixed the false CI sentence; refreshed the test-list parenthetical (howto+spec stale-checks, skill-determinism, compare-parity, sync-goldens, envelope-smoke); added the `Data`-variant↔`rows()` guidance. Build block, layering, source-of-truth, runtime, IDs, errors, comments conventions all verified current.
- Test count: 166 + 2 ignored + 2 xtask (was 164+2-ignored+2 at P9 exit; +2 = the two envelope tests).
- ⏳ **P5 "sync-preserving `update`" still deferred** (YAGNI — reaffirmed P5/P8/P9/P10).

## Phase 11 — Scenario examples (multi-command workflows)

- [x] `docs/design/18-scenarios.md` + `docs/adr/013-compile-enforced-scenarios.md` authored
- [x] `examples/` dir (repo root, `.md`-only) + first scenario file (`examples/build-a-course.md`)
- [x] `build.rs`: parse `examples/*.md` fenced ```sh blocks → distinct `<group>::<fn>`
      count; assert **≥3** per file + **≥1** file exists; reuse the existing command-fn
      set (no new attribute — the "tag" *is* the `pub fn -> Result<Data, _>` signature);
      `cargo:rerun-if-changed=examples`
- [x] `xtask/src/howto.rs`: `collect_scenarios()` (sorted by filename for determinism) →
      append `## Scenarios` section to `howto.gen.md` (verbatim, headings demoted +2)
- [x] Regenerate `src/howto.gen.md` (via `cargo xtask gen-howto`)
- [x] `core/skill.rs`: scenarios auto-inline into `SKILL.md` via existing `render()`
      (free — `render()` already inlines `manual::MANUAL`); added one authored pointer
      sentence so the agent is directed to the inlined scenarios
- [x] **Tests:** howto stale-check covers the new section (free) + new
      `howto_includes_scenarios`; build.rs gate verified to fire on <3 distinct fns
      (manual negative test); `frontmatter_validates` extended for the scenarios pointer
- [x] Phase-exit gate green (`cargo xtask build` + `cargo test --workspace` 167+2-ignored+3 + clippy/fmt/doc `--workspace`)

**Notes (P11):**
- "Tag" on a public fn = its existing signature (`pub fn -> Result<Data, _>` in `commands/`); no proc-macro (adr/013 rejected it, same as adr/007).
- Parser normalizes leading globals (`-c X` / `--course X` / `--root P`, value-skipping; `--flag=val` and boolean globals like `--version` handled) before reading `<group> <fn>`; the same fn invoked twice counts once (distinct). Unknown invocations fail the build by name. Only ```sh/```bash fences are parsed — ```json envelopes are never counted. Top-level commands (no group) resolve as `<name>::<name>`.
- The collector **demotes scenario headings by +2** (`# `→`### `, `## `→`#### `, capped at H6, fence-aware) so a scenario nests cleanly under `## Scenarios` — same "mechanical shift, no prose rewrite" spirit as `skill::manual_body()` stripping the manual's H1. Content is otherwise byte-verbatim.
- First scenario `examples/build-a-course.md` distills a real agent session (a 16-lesson linear-algebra course) into the canonical plan-first flow: `course → venv → plan create → plan confirm → lesson create → lesson execute → quiz run → progress summary` (9 distinct fns). Includes a minimal inline `lesson.json` stub + a Conventions section (strict-`==` answer-key check, round floats to 8dp, sign-free cases).
- Cargo `examples/` auto-discovery targets only `*.rs`; `.md`-only is harmless (note in design/18; set `autoexamples = false` if a stray `.rs` ever lands).
- Scenarios do **not** appear in `docs/specs/` — those are per-command envelope contracts; scenarios are howto material.
- Skill inclusion is mechanical duplication (the howto is still the single authored source via `xtask gen-howto`); the authored pointer sentence is the one new atom (adr/009 style). `carpenter register --print-skill` confirms `## Scenarios` + `### Build a course end-to-end` + the pointer all land in the rendered skill (38.6 KB body).

---

## Dependency chain

```
P0 gates ──> P1 codegen ──> P2 db/course ──> P3 plan/goal
                                │
                                └─> P4 lesson authoring (notebook+helper+compare) ──> P5 sync/lifecycle
                                                                                       │
                                                                                       └─> P6 execute (venv+nbconvert) ──> P7 skip ──> P8 progress/notes ──> P9 meta ──> P10 hardening ──> P11 scenarios
```
