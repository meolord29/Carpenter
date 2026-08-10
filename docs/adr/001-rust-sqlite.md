# ADR-001: Rust + SQLite

Date: 2026-08-08 · Status: Accepted

## Context
carpenter must be (a) callable globally from a `bin/` directory, (b) self-documenting
via a build-time scrape of its command surface, and (c) a store for a relational
model (course → lesson → section/practice/quiz → test cases) with joins. An earlier
sketch used Python (typer/pydantic/nbformat/tinydb), but the requirements for a
distributable compiled binary and a build-time documentation step pointed away from
Python, and "TinyDB" was a placeholder for "local document store."

## Decision
Implement carpenter in **Rust** (`clap`, `serde`, `rusqlite`, `thiserror`,
`serde_json` for notebooks) with **SQLite** as the per-course store. Learner code is
still Python and is executed via `std::process::Command` in an isolated subprocess.

## Consequences
+ Single distributable binary; trivial global install via `install`.
+ Native schema, foreign keys, and JSON1 for the joins — strictly better fit than a
  flat document store.
+ `clap`'s compiled command tree is a clean scrape target for `howto` (ADR-003).
+ Generated `helper.py` reads SQLite via Python stdlib `sqlite3` — learner needs no
  extra dependencies.
− Two languages in play: Rust for the CLI, Python for `helper.py` and learner code.
  Compare logic exists in both; semantics are locked in `docs/specs/` and asserted
  by tests in each language.
− No TinyDB specifically; "tinydb as a document" maps to SQLite rows / JSON columns.
