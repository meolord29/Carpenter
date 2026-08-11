# ADR-014: YAML is the spec format

Date: 2026-08-11 · Status: Accepted (supersedes the earlier "JSON or YAML" version)

## Context

`--spec` commands (`course create`, `plan create`, `lesson create`, `goal add`,
`notes add`, `bug file`, `feature file`, `lesson verify`) author specs. JSON was
the original format. JSON is painful for lesson/plan specs whose `content` fields
hold multi-line markdown (with LaTeX) and code: every newline and quote must be
escaped (`\n`, `\"`, `\\`), so an authoring agent reached for a **Python generator
script** to emit the nested JSON rather than authoring the spec directly — which
also introduced encoding bugs impossible in a strict-UTF-8 read path.

Authors (humans and agents) should author specs directly, with native multi-line
strings, and the CLI should offer one format — not two to choose between.

## Decision

**YAML is the single spec format.** `parse_spec` (`core/store.rs`) calls
`serde_yml::from_str` — one parser, no JSON code path. Block scalars (`|`) give
native multi-line `content`/`solution` without escaping.

- **No backward compatibility.** A JSON-syntax spec is accepted only when it
  happens to be valid YAML flow syntax (YAML is a superset of JSON, so
  `{"title":"x"}` parses as a flow mapping) — there is no separate JSON branch
  and no guarantee JSON parses. Authors write YAML.
- **Crate:** `serde-yml` (maintained fork; `serde_yaml` was archived in 2024).
- **Read path unchanged:** `read_spec` still uses strict-UTF-8 `read_to_string`
  (file or stdin); YAML parsing happens after read, on a Rust `String`.
- **`serde_json` stays in the crate** for everything that is NOT an authored spec:
  the output envelope (`core/output.rs`), `course.json`/`config.json`/issue
  files (carpenter-written, see scope), DB JSON columns, and notebooks
  (`.ipynb`, nbformat-locked). Output envelopes remain JSON — they are the
  machine-readable agent contract, not authored files.

## Consequences

+ One spec format; native multi-line strings; no Python-generator workaround and
  no encoding-bug class. The worked examples (`docs/examples/`, the scenario, and
  the generated `docs/specs/` tables) all author YAML.
+ Single parser path — simpler than the prior try-JSON-then-YAML.

− **YAML scalar quoting discipline** falls on the author: any value containing
  `: ` (colon + space) or ending in `:` must be quoted. Lesson specs hit this
  constantly — every `signature` ends in `:` (e.g. `def f(x):`), so YAML specs
  must quote signatures: `signature: "def f(x):"`. `lesson new` emits a template
  showing this; it is inherent to YAML, not enforced.
− YAML 1.1 bool gotcha (`yes`/`no`/`on`) is mitigated by `serde-yml`'s YAML 1.2
  core schema (only `true`/`false`), but authors should quote ambiguous strings.
− One new dependency (`serde-yml` + transitive `indexmap`/`libyaml`-class parser).

## Scope (what is NOT YAML)

This decision covers **authored `--spec` inputs only**. The following remain JSON
deliberately: command output envelopes (stdout), `lesson.ipynb` (nbformat v4),
DB JSON columns (`snippets`, `last_check`, `args`, `kwargs`, `expected`, `tags`,
`covered_by`), `opencode.json` (external/opencode-owned), and the carpenter-
written on-disk files `course.json` / `config.json` / `bug|feature_request/*.json`
+ the verify internal `payload.json`. Converting those is a separate decision
with no authoring-ergonomics payoff.
