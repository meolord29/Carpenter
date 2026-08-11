//! The generated `verify.py` — author answer-key self-check (stdlib-only).
//!
//! Mirrors `core/compare.rs` via the same `_compare` as `helper.rs`. Runs each
//! author reference `solution` against its own cases and reports per-case
//! pass/fail. **Never prints `expected`** ([adr/015](../../docs/adr/015-reference-solution-verify.md)).
//!
//! Unlike `helper.py`, this is NOT written beside the notebook — `lesson verify`
//! stages it in a temp dir and runs it via `uv run` in the course venv. It reads
//! its payload (solutions + cases) from a JSON file path passed as `argv[1]`, so it
//! never touches `course.db` and works for both `--spec` (pre-create) and `<id>`
//! (post-create) modes.

/// The full `verify.py` source.
pub const VERIFY_PY: &str = r#"# carpenter verify script (generated; do not edit).
#
# Stdlib-only. Runs each author reference solution against its own cases and
# reports per-case pass/fail. Mirrors helper.py's _compare (== core/compare.rs).
# NEVER prints `expected`.

import json
import signal
import sys


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


class _Timeout(Exception):
    pass


def _alarm(signum, frame):
    raise _Timeout("timeout")


# Per-case wall-clock guard. SIGALRM is Unix-only; on Windows there is no hard
# kill (best-effort), matching nbconvert's cross-platform behavior.
_has_alarm = hasattr(signal, "SIGALRM")
if _has_alarm:
    signal.signal(signal.SIGALRM, _alarm)


def _run_case(fn, case, timeout):
    """Return (passed, actual_repr_or_None, error_or_None). Never returns expected."""
    args = case.get("args", [])
    kwargs = case.get("kwargs", {})
    expected = case["expected"]
    mode = case.get("compare", "exact")
    if _has_alarm:
        signal.alarm(timeout)
    try:
        got = fn(*args, **kwargs)
        try:
            ok = _compare(got, expected, mode)
        except _CompareError as e:
            return False, None, str(e)
        return (True, None, None) if ok else (False, repr(got), None)
    except _Timeout:
        return False, None, "timeout"
    except Exception as e:  # noqa: BLE001 — author errors are scored as fails
        return False, None, type(e).__name__ + ": " + str(e)
    finally:
        if _has_alarm:
            signal.alarm(0)


def main():
    payload = json.load(open(sys.argv[1]))
    timeout = int(payload.get("timeout", 30))
    out = []
    for chk in payload.get("checkables", []):
        name = chk["name"]
        solution = chk.get("solution", "")
        cases = chk.get("cases", [])
        ns = {}
        load_error = None
        if solution:
            try:
                exec(compile(solution, "<solution>", "exec"), ns)
            except Exception as e:  # noqa: BLE001
                load_error = type(e).__name__ + ": " + str(e)
        fn = ns.get(name) if (solution and not load_error) else None
        case_results = []
        passed = 0
        for c in cases:
            if not solution:
                ok, actual, err = False, None, "no solution"
            elif load_error:
                ok, actual, err = False, None, "solution load error: " + load_error
            elif fn is None:
                ok, actual, err = False, None, "solution did not define " + name
            else:
                ok, actual, err = _run_case(fn, c, timeout)
            entry = {"case_id": c.get("case_id"), "passed": bool(ok)}
            if actual is not None:
                entry["actual"] = actual
            if err is not None:
                entry["error"] = err
            case_results.append(entry)
            passed += 1 if ok else 0
        out.append({
            "owner_type": chk.get("owner_type"),
            "owner_id": chk.get("owner_id"),
            "name": name,
            "has_solution": bool(solution),
            "passed": passed,
            "total": len(cases),
            "cases": case_results,
        })
    print(json.dumps({"checkables": out}))


if __name__ == "__main__":
    main()
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_has_required_surface() {
        assert!(VERIFY_PY.contains("def main("));
        assert!(VERIFY_PY.contains("def _compare("));
        assert!(VERIFY_PY.contains("def _run_case("));
        assert!(VERIFY_PY.contains("has_solution"));
        // same invariant as helper: NEVER prints expected.
        assert!(!VERIFY_PY.contains("print(expected"));
        // the result envelope carries `actual` (repr) and `error`, not `expected`.
        assert!(VERIFY_PY.contains("\"actual\""));
        assert!(!VERIFY_PY.contains("expected_j"));
    }

    #[test]
    fn verify_is_valid_python() {
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
            let _ = stdin.write_all(VERIFY_PY.as_bytes());
        }
        let status = child.wait().expect("wait");
        assert!(status.success(), "verify.py is not valid Python");
    }
}
