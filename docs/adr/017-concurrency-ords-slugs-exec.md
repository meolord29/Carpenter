# ADR-017: concurrency semantics — atomic ords, slug validation, serialized execution

Date: 2026-08-17 · Status: Accepted

## Context

A black-box QA pass (agent-driven corpus build of a 16-lesson course) found
three concurrency/validation defects, all silent or fatal:

1. **Duplicate `lesson.ord` under concurrent `lesson create`.** The next ord
   was computed by a standalone `SELECT MAX(ord)` read followed by a separate
   INSERT — a check-then-insert gap. SQLite serializes *statements*, not
   statement *pairs*, so concurrent creators observed the same `MAX` snapshot
   and inserted the same ord (observed: ord 1 ×3, 7 ×4, 9 ×2 in one course;
   three lesson dirs sharing an `01-` prefix). Nothing in the schema forbade
   it, and nothing surfaced it.
2. **Concurrent `quiz run` / `lesson execute` died on a kernel-port race.**
   carpenter pins no ports; the collision is jupyter_client's
   bind-probe-release port reservation (a TOCTOU across processes sharing one
   runtime dir). The losing ipykernel died with `ZMQError: Address already in
   use` and nbclient burned its default 60 s startup timeout before failing.
3. **Provided slugs were accepted verbatim.** `slugify`'s ASCII gate ran only
   when a slug was *absent*; a non-ASCII provided slug was persisted as both a
   directory name and a DB row. On a Unicode-normalizing filesystem (macOS
   APFS stores NFD) the dir name diverges from the NFC DB slug, and the course
   stops resolving by the documented byte-equality lookups.

## Decision

1. **Atomic ord allocation + unique index.** The auto path allocates
   `max(ord)+1` *inside* the INSERT statement
   (`VALUES (…, (SELECT COALESCE(MAX(ord),0)+1 FROM lessons)) RETURNING ord`)
   — statement-level serialization makes the pair race-free (the same trick
   `id_seq`/`next_id` already uses). A guarded migration rese sequences legacy
   duplicate ords densely (stable by `ord, created_at, id`) and creates
   `CREATE UNIQUE INDEX idx_lessons_ord ON lessons(ord)`. An explicit
   `spec.order` colliding with an existing lesson's ord is a `Conflict`
   (`"lesson ord N already taken"`).
2. **Serialize notebook execution per course** with an exclusive advisory
   lock (`.exec.lock` flock in the course dir; fs4). flock releases on process
   death, so a crashed run leaves no stale lock. Waiters block up to 120 s,
   then fail with a clear StoreError naming the concurrent run. Chosen over
   env-isolation+retry: a lock makes the invariant structural (two kernels in
   one course never launch simultaneously) instead of probabilistic, and the
   secondary shared state (both kernels' `helper.check` write the one
   `course.db`) is de-facto serialized too.
3. **Validate provided slugs** against the documented slug convention
   (`docs/data-model/02-conventions.md`): `^[a-z0-9]+(-[a-z0-9]+)*$`, ≤ 60
   chars, else `ValidationError` naming the rule. Applied where a slug enters
   the system (`course create`, `lesson create`; updates never change slugs).
   Rejected auto-normalization (`slugify` the input silently) — mutating
   author input without consent hides errors; derived slugs (title → slugify)
   already satisfy the rule by construction.

### Alternatives rejected

- **`BEGIN IMMEDIATE` around read+insert** for ords: works, but adds the
  first transaction to a codebase that is deliberately autocommit-simple, for
  what one statement can do.
- **Per-run `JUPYTER_RUNTIME_DIR` + bounded retry** for the port race: keeps
  parallelism but leaves a (tiny) residual collision window and a retry path
  that is untestable without a real Jupyter install; the lock is testable and
  total.
- **Slug auto-normalization**: silently rewriting `Bad_Slug` → `bad-slug`
  trades a loud, fixable ValidationError for a quiet surprise.

## Consequences

+ Concurrent `lesson create`s get distinct ords by construction; the unique
  index turns any future violation (hand-edited DB, exotic interleaving) into
  a loud conflict instead of a silent duplicate.
+ Legacy dirty DBs self-heal on first open (resequence + index).
+ Concurrent `quiz run`/`lesson execute` within one course queue instead of
  dying after a 60 s hang. Cross-course parallelism is unaffected (lock is
  per course dir).
+ Non-ASCII slugs are rejected at the door with a rule the author can act on;
  directory-name ↔ DB-slug divergence becomes impossible.
+ − Same-course execution no longer runs in parallel — a long notebook blocks
  other executions in that course for up to 120 s before erroring. Accepted:
  course-local scoring is minutes-scale, and correctness (kernel + single
  `course.db` writers) outranks throughput here.
+ − The explicit `order` escape hatch now *conflicts* on duplicates instead of
  silently double-assigning — authors pinning orders must pin distinct ones.
