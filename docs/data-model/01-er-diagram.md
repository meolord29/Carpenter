# Entity-relationship

```
course_meta (1 row; mirrors course.json)   <-- distinct from `goals` rows
  │
  ├──< lessons (slug, ordered; status derived from pass_or_fail+skip)
  │      │
  │      ├──< sections (ordered; snippets: markdown+code -> cells)
  │      │      │
  │      │      └──< practice (Checkable; pass_or_fail, last_check, skip)
  │      │             │
  │      │             └──< test_cases (owner_type=practice)   [* polymorphic: no FK *]
  │      │
  │      └──< quizzes (Checkable, end of lesson; pass_or_fail, last_check, skip)
  │             │
  │             └──< test_cases (owner_type=quiz)              [* polymorphic: no FK *]
  │
  ├──< notes      (qualitative; optional related lesson/quiz — free ref, no FK)
  ├──< plans      (scope=course|lesson, a document)
  └──< goals      (scope=course; covered_by -> lessons[] JSON; override flag)
```

Notes:
- The `attempts` table was removed (adr/010). Live check state lives on
  `practice`/`quizzes` (`pass_or_fail`, `last_check`); there is no history.
- `course_meta.goal` (singular mission statement, prose) is **distinct** from the
  `goals` table (granular objective rows).
- `test_cases.owner_id`, `notes.related`, `plans.scope_id`, `goals.covered_by` are
  **polymorphic / free refs with no FK** — orphan cleanup is the application's job.
