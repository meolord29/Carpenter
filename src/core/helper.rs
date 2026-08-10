//! The generated, generic `helper.py` (verification-only, stdlib-only).
//!
//! Identical for every lesson; never embeds cases (reads them by id from
//! `course.db`). Mirrors `core/compare.rs` semantics and writes back
//! `pass_or_fail`/`last_check` after a check. **Never prints `expected`.**

/// The full `helper.py` source, written alongside each lesson notebook.
pub const HELPER_PY: &str = r#"# carpenter verification helper (generated; do not edit).
#
# Stdlib-only. Verification-only: scores a function against the owner's test
# cases read from course.db and writes back pass_or_fail/last_check.
# NEVER prints `expected`.

import json
import sqlite3
from pathlib import Path

_DB = Path(__file__).resolve().parents[2] / "course.db"


class _CompareError(Exception):
    pass


def _compare(actual, expected, mode):
    if mode == "exact":
        return actual == expected
    if mode == "sorted":
        try:
            return sorted(actual) == sorted(expected)
        except TypeError:
            raise _CompareError("unsortable")
    if mode == "set":
        try:
            return set(actual) == set(expected)
        except TypeError:
            raise _CompareError("unhashable")
    return False


_TABLE = {"practice": "practice", "quiz": "quizzes"}


def check(owner_type, owner_id, fn):
    """Score `fn` against the owner's cases; write pass_or_fail + last_check."""
    conn = sqlite3.connect(str(_DB))
    try:
        cases = conn.execute(
            "SELECT id, args, kwargs, expected, compare FROM test_cases "
            "WHERE owner_type=? AND owner_id=? ORDER BY ord",
            (owner_type, owner_id),
        ).fetchall()
        results = []
        passed = 0
        total = len(cases)
        for case_id, args_j, kwargs_j, expected_j, mode in cases:
            args = json.loads(args_j)
            kwargs = json.loads(kwargs_j)
            err = None
            try:
                got = fn(*args, **kwargs)
                ok = _compare(got, json.loads(expected_j), mode)
            except _CompareError as e:
                ok, err = False, str(e)
            except Exception as e:  # noqa: BLE001 — learner errors are scored as fails
                ok, err = False, type(e).__name__ + ": " + str(e)
            entry = {"case_id": case_id, "passed": 1 if ok else 0}
            if err is not None:
                entry["error"] = err
            results.append(entry)
            passed += 1 if ok else 0
            print(("PASS" if ok else "FAIL"), owner_id, case_id)
        print("%d/%d" % (passed, total))
        conn.execute(
            "UPDATE " + _TABLE[owner_type] + " SET pass_or_fail=?, last_check=? WHERE id=?",
            (
                1 if (total > 0 and passed == total) else 0,
                json.dumps({"passed": passed, "total": total, "cases": results}),
                owner_id,
            ),
        )
        conn.commit()
    finally:
        conn.close()


def is_skipped(target):
    """True if `target` (a practice/quiz id or the lesson slug) is skipped."""
    conn = sqlite3.connect(str(_DB))
    try:
        if target.startswith("p"):
            row = conn.execute(
                "SELECT skip FROM practice WHERE id=?", (target,)
            ).fetchone()
        elif target.startswith("q"):
            row = conn.execute(
                "SELECT skip FROM quizzes WHERE id=?", (target,)
            ).fetchone()
        else:
            row = conn.execute(
                "SELECT skip FROM lessons WHERE id=?", (target,)
            ).fetchone()
        return bool(row and row[0])
    finally:
        conn.close()
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_has_required_surface() {
        assert!(HELPER_PY.contains("def check("));
        assert!(HELPER_PY.contains("def is_skipped("));
        assert!(HELPER_PY.contains("def _compare("));
        assert!(HELPER_PY.contains("parents[2]"));
        // verification-only: it writes pass_or_fail/last_check, never the expected value
        assert!(HELPER_PY.contains("pass_or_fail"));
        assert!(!HELPER_PY.contains("print(expected"));
    }

    #[test]
    fn helper_is_valid_python() {
        use std::process::Command;
        let Ok(out) = Command::new("python3")
            .arg("-c")
            .arg("import sys, ast; ast.parse(sys.stdin.read())")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
        else {
            return;
        };
        use std::io::Write;
        let mut child = out;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(HELPER_PY.as_bytes());
        }
        let status = child.wait().expect("wait");
        assert!(status.success(), "helper.py is not valid Python");
    }
}
