# Build order

1. Skeleton: `Cargo.toml` (workspace: `carpenter` + `xtask`); `app.rs` + `_emit`
   + `output` + `error` + `store`; clap wiring for a couple of commands. Add
   `#![deny(missing_docs)]` at the crate root and the `build.rs` syn scanner
   ([adr/007](../adr/007-compile-enforced-command-docs.md)) on day one — so the gates
   are in place before any real command lands.
2. `xtask gen-howto` → `howto.gen.md` + `howto` command; `xtask gen-specs` →
   `docs/specs/*.md` from the `*Spec`/`Data` types + their co-located `mod examples`
   ([adr/008](../adr/008-specs-generated-from-types.md)). Establish both scraping
   pipelines early. (Spec marker regions are added in this phase; until then the
   spec tables are hand-maintained — see [specs/README.md](../specs/README.md).)
3. Course CRUD: `db.rs` (initial schema as one `CREATE` migration — a full versioned
   `migrate` command is deferred per adr/010), course commands.
4. Plans + goals (human-in-the-loop confirm; status derivation).
5. Lesson authoring: `LessonSpec`, `lesson create` → DB + render (managed cells
   incl. skip-config) + helper.
6. Lesson lifecycle: `sync` (3-way preservation via `scaffold_hash`), get/list/show/delete.
7. venv (`uv`) + `lesson execute` (nbconvert) + `quiz run` (nbconvert + helper
   writes `pass_or_fail`/`last_check`); quiz list/show/results.
8. `skip` command (sets `skip` columns, drives status derivation + render).
9. Progress + notes (summary, `related_open` hint).
10. Meta commands: bug, feature, config, link (manifest), build, install, upgrade,
    register/deregister (skill integration).
11. Envelope smoke test across all commands; `AGENTS.md` finalized.
12. Cross-platform (Linux/macOS/Windows): [adr/012](../adr/012-cross-platform-paths.md),
    [design/17](17-cross-platform.md). `core/platform.rs` (`#[cfg(target_os)]`:
    `default_bin_dir` + `exe_file_name`); wire `Config::default` + `install`/`upgrade` to
    it. Normalize the 8 `String::ends_with` path assertions to `Path`-component checks.
    Add `.gitattributes` (`eol=lf`) so the byte-equality stale-checks hold on Windows.
    Add `.github/workflows/ci.yml` (3-OS matrix, full gate suite, `uv` installed) +
    `rust-toolchain.toml` (stable pin). `config_dir` already portable via `dirs` — no
    change there.
13. Scenario examples ([design/18](18-scenarios.md),
    [adr/013](../adr/013-compile-enforced-scenarios.md)): a second `build.rs` gate
    over `examples/*.md` (each scenario must reference ≥3 distinct command fns; ≥1
    must exist); `xtask gen-howto` appends a `## Scenarios` section (verbatim) →
    auto-inlined into `SKILL.md` by `render()`.
