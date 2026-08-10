# ADRs

Architecture decisions — context, decision, consequences. Numbered; gaps in the
sequence are retired decisions (not listed). Read topically; start with the index.

| # | file | topic |
|---|------|-------|
| 001 | [001-rust-sqlite.md](001-rust-sqlite.md) | Rust binary + SQLite store, no embedded LLM |
| 002 | [002-db-source-of-truth.md](002-db-source-of-truth.md) | DB is the source of truth; notebooks are rendered |
| 003 | [003-howto-buildstep.md](003-howto-buildstep.md) | howto generated at build time from the clap surface |
| 004 | [004-build-install-split.md](004-build-install-split.md) | build / install / upgrade split |
| 006 | [006-skill-integration.md](006-skill-integration.md) | integrate via a SKILL.md, not custom tools |
| 007 | [007-compile-enforced-command-docs.md](007-compile-enforced-command-docs.md) | compile-enforced command self-documentation (missing_docs + build.rs scan) |
| 008 | [008-specs-generated-from-types.md](008-specs-generated-from-types.md) | specs generated from `*Spec`/`Data` types (region-with-markers) |
| 009 | [009-skill-assembled-from-fields.md](009-skill-assembled-from-fields.md) | SKILL.md assembled from code fields, no template |
| 010 | [010-live-check-state.md](010-live-check-state.md) | live check state via `pass_or_fail` (no `attempts` table); helper rw |
| 011 | [011-skip-db-authored.md](011-skip-db-authored.md) | skip is DB-authored + rendered (not notebook-parsed) |
