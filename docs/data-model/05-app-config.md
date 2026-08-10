# App-level config (`~/.config/carpenter/`)

Not per-course. Lives under the XDG config dir.

```
config.json                # {bin_dir, python, timeout_secs, active_course?}
bug/<id>.json              # {id,ts,title,description,repro,stack?,status,resolved_ts?}
feature_request/<id>.json  # {id,ts,title,description,rationale,status,resolved_ts?}
```
- `config.json` keys + types + defaults: see [specs/16-config.md](../specs/16-config.md).
- `status ∈ {open, resolved}`.
- `<id>` = `<prefix><N>` (`b1`,`b2`… for bug; `f1`,`f2`… for feature), `max+1` per
  kind, never reused.
- `stack?` is server-added (a captured traceback), not part of `BugSpec`.
