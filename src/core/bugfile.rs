//! File-backed bug/feature store (`~/.config/carpenter/{bug,feature_request}/`).
//!
//! Each issue is one JSON file `<prefix><N>.json` (`b1`… for bug, `f1`… for
//! feature); ids are `max+1` per kind over existing files, never reused. No
//! SQLite — app-level only ([data-model/05](../../docs/data-model/05-app-config.md)).

use std::path::{Path, PathBuf};

use crate::core::error::CarpenterError;
use crate::core::store;
use crate::core::time;
use crate::models::common::RowError;
use crate::models::issue::{IssueListItem, IssueSpec};

/// Which kind of issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A bug (`bug/`, prefix `b`).
    Bug,
    /// A feature request (`feature_request/`, prefix `f`).
    Feature,
}

impl Kind {
    /// The subdirectory under the config dir.
    pub fn dir(self) -> &'static str {
        match self {
            Kind::Bug => "bug",
            Kind::Feature => "feature_request",
        }
    }

    /// The id prefix.
    pub fn prefix(self) -> &'static str {
        match self {
            Kind::Bug => "b",
            Kind::Feature => "f",
        }
    }

    /// Human label for messages.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Bug => "bug",
            Kind::Feature => "feature",
        }
    }
}

/// The full stored record (union of bug + feature fields; unused ones `None`).
#[derive(Debug, Clone)]
pub struct IssueRecord {
    /// id.
    pub id: String,
    /// created timestamp.
    pub ts: String,
    /// title.
    pub title: String,
    /// description.
    pub description: String,
    /// repro (bug only).
    pub repro: Option<String>,
    /// rationale (feature only).
    pub rationale: Option<String>,
    /// captured stack trace (bug only, server-added; `None` on file).
    pub stack: Option<String>,
    /// `open` | `resolved`.
    pub status: String,
    /// set when resolved.
    pub resolved_ts: Option<String>,
}

/// Validate an [`IssueSpec`] for `kind`:
/// - title/description non-empty;
/// - bug ⇒ `repro` allowed, `rationale` rejected;
/// - feature ⇒ `rationale` allowed, `repro` rejected.
pub fn validate(kind: Kind, spec: &IssueSpec) -> Result<(), CarpenterError> {
    if spec.title.trim().is_empty() {
        return Err(CarpenterError::ValidationError(
            "title must be non-empty".into(),
        ));
    }
    if spec.description.trim().is_empty() {
        return Err(CarpenterError::ValidationError(
            "description must be non-empty".into(),
        ));
    }
    match kind {
        Kind::Bug => {
            if spec.rationale.is_some() {
                return Err(CarpenterError::ValidationError(
                    "rationale is feature-only (use `feature file`)".into(),
                ));
            }
        }
        Kind::Feature => {
            if spec.repro.is_some() {
                return Err(CarpenterError::ValidationError(
                    "repro is bug-only (use `bug file`)".into(),
                ));
            }
        }
    }
    Ok(())
}

fn kind_dir(config_dir: &Path, kind: Kind) -> PathBuf {
    config_dir.join(kind.dir())
}

fn file_path(config_dir: &Path, kind: Kind, id: &str) -> PathBuf {
    kind_dir(config_dir, kind).join(format!("{id}.json"))
}

/// Allocate the next id for `kind` (`max+1` over existing `<prefix><N>.json`).
pub fn next_id(config_dir: &Path, kind: Kind) -> Result<String, CarpenterError> {
    let dir = kind_dir(config_dir, kind);
    let mut max = 0u64;
    if dir.exists() {
        for entry in std::fs::read_dir(&dir).map_err(store::io_to_store)? {
            let Ok(entry) = entry else { continue };
            let Some(num) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.strip_prefix(kind.prefix()))
                .and_then(|s| s.strip_suffix(".json"))
                .and_then(|s| s.parse::<u64>().ok())
            else {
                continue;
            };
            if num > max {
                max = num;
            }
        }
    }
    Ok(format!("{}{}", kind.prefix(), max + 1))
}

/// Build the stored JSON value for a record.
fn to_json(kind: Kind, rec: &IssueRecord) -> serde_json::Value {
    match kind {
        Kind::Bug => serde_json::json!({
            "id": rec.id,
            "ts": rec.ts,
            "title": rec.title,
            "description": rec.description,
            "repro": rec.repro,
            "stack": rec.stack,
            "status": rec.status,
            "resolved_ts": rec.resolved_ts,
        }),
        Kind::Feature => serde_json::json!({
            "id": rec.id,
            "ts": rec.ts,
            "title": rec.title,
            "description": rec.description,
            "rationale": rec.rationale,
            "status": rec.status,
            "resolved_ts": rec.resolved_ts,
        }),
    }
}

fn str_field(v: &serde_json::Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(String::from)
}

/// Parse a stored JSON object into an [`IssueRecord`].
fn parse_record(kind: Kind, id: &str, v: &serde_json::Value) -> Result<IssueRecord, String> {
    let title = str_field(v, "title").ok_or_else(|| format!("missing title for {id}"))?;
    let (repro, rationale, stack) = match kind {
        Kind::Bug => (str_field(v, "repro"), None, str_field(v, "stack")),
        Kind::Feature => (None, str_field(v, "rationale"), None),
    };
    Ok(IssueRecord {
        id: id.into(),
        ts: str_field(v, "ts").unwrap_or_default(),
        title,
        description: str_field(v, "description").unwrap_or_default(),
        repro,
        rationale,
        stack,
        status: str_field(v, "status").unwrap_or_else(|| String::from("open")),
        resolved_ts: str_field(v, "resolved_ts"),
    })
}

/// File a new issue; returns the new id + absolute path.
pub fn file(
    config_dir: &Path,
    kind: Kind,
    spec: &IssueSpec,
) -> Result<(String, String), CarpenterError> {
    validate(kind, spec)?;
    let id = next_id(config_dir, kind)?;
    let rec = IssueRecord {
        id: id.clone(),
        ts: time::now_iso(),
        title: spec.title.clone(),
        description: spec.description.clone(),
        repro: if matches!(kind, Kind::Bug) {
            spec.repro.clone()
        } else {
            None
        },
        rationale: if matches!(kind, Kind::Feature) {
            spec.rationale.clone()
        } else {
            None
        },
        stack: None,
        status: String::from("open"),
        resolved_ts: None,
    };
    let path = file_path(config_dir, kind, &id);
    store::atomic_write(&path, to_json(kind, &rec).to_string().as_bytes())?;
    Ok((id, path.display().to_string()))
}

/// List all issues of `kind` (id/title/status), sorted by id; corrupt files
/// surface in `errors[]` (id from the filename) — never silently dropped.
pub fn list(
    config_dir: &Path,
    kind: Kind,
) -> Result<(Vec<IssueListItem>, Vec<RowError>), CarpenterError> {
    let dir = kind_dir(config_dir, kind);
    let mut items = Vec::new();
    let mut errors = Vec::new();
    if !dir.exists() {
        return Ok((items, errors));
    }
    for entry in std::fs::read_dir(&dir).map_err(store::io_to_store)? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let Some(id) = path.file_stem().and_then(|s| s.to_str()).map(String::from) else {
            continue;
        };
        match std::fs::read_to_string(&path).ok() {
            None => errors.push(RowError {
                id: Some(id),
                reason: String::from("unreadable file"),
            }),
            Some(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(v) => match parse_record(kind, &id, &v) {
                    Ok(rec) => items.push(IssueListItem {
                        id: rec.id,
                        title: rec.title,
                        status: rec.status,
                    }),
                    Err(reason) => errors.push(RowError {
                        id: Some(id),
                        reason,
                    }),
                },
                Err(e) => errors.push(RowError {
                    id: Some(id),
                    reason: format!("corrupt json: {e}"),
                }),
            },
        }
    }
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok((items, errors))
}

/// Read one issue as a full record. `NotFound` if the file is absent.
pub fn show(config_dir: &Path, kind: Kind, id: &str) -> Result<IssueRecord, CarpenterError> {
    let path = file_path(config_dir, kind, id);
    read_record(&path, kind, id)
}

/// Resolve an issue (`status='resolved'`, stamp `resolved_ts`). `NotFound` if absent.
/// Returns the `resolved_ts`.
pub fn resolve(config_dir: &Path, kind: Kind, id: &str) -> Result<String, CarpenterError> {
    let path = file_path(config_dir, kind, id);
    let text = std::fs::read_to_string(&path)
        .map_err(|_| CarpenterError::NotFound(format!("{} {id}", kind.label())))?;
    let mut v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| CarpenterError::StoreError(format!("corrupt issue file {id}: {e}")))?;
    let now = time::now_iso();
    if let Some(obj) = v.as_object_mut() {
        obj.insert(
            String::from("status"),
            serde_json::Value::String("resolved".into()),
        );
        obj.insert(
            String::from("resolved_ts"),
            serde_json::Value::String(now.clone()),
        );
    }
    store::atomic_write(&path, v.to_string().as_bytes())?;
    Ok(now)
}

fn read_record(path: &Path, kind: Kind, id: &str) -> Result<IssueRecord, CarpenterError> {
    let text = std::fs::read_to_string(path)
        .map_err(|_| CarpenterError::NotFound(format!("{} {id}", kind.label())))?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| CarpenterError::StoreError(format!("corrupt issue file {id}: {e}")))?;
    parse_record(kind, id, &v).map_err(CarpenterError::StoreError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::issue::IssueSpec;

    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn cfg_dir() -> PathBuf {
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let d = std::env::temp_dir().join(format!("carpenter-bugfile-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn bug_spec() -> IssueSpec {
        IssueSpec {
            title: String::from("crash"),
            description: String::from("it crashes"),
            repro: Some(String::from("run x")),
            rationale: None,
        }
    }

    fn feature_spec() -> IssueSpec {
        IssueSpec {
            title: String::from("add thing"),
            description: String::from("want it"),
            repro: None,
            rationale: Some(String::from("because")),
        }
    }

    #[test]
    fn next_id_is_monotonic_across_files() {
        let d = cfg_dir();
        // empty dir → b1 (no files written yet)
        assert_eq!(next_id(&d, Kind::Bug).expect("id"), "b1");
        // filing advances max+1 over the written files; strictly increasing
        let (id1, _) = file(&d, Kind::Bug, &bug_spec()).expect("file");
        let (id2, _) = file(&d, Kind::Bug, &bug_spec()).expect("file");
        let (id3, _) = file(&d, Kind::Bug, &bug_spec()).expect("file");
        assert_eq!(
            (id1.as_str(), id2.as_str(), id3.as_str()),
            ("b1", "b2", "b3")
        );
        // there is no `delete` command, so ids never go backwards in practice
        assert_eq!(next_id(&d, Kind::Bug).expect("id"), "b4");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn next_id_is_independent_per_kind() {
        let d = cfg_dir();
        file(&d, Kind::Bug, &bug_spec()).expect("bug");
        file(&d, Kind::Bug, &bug_spec()).expect("bug");
        let (f, _) = file(&d, Kind::Feature, &feature_spec()).expect("feature");
        assert_eq!(f, "f1", "feature ids start at f1 regardless of bugs");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn validate_branches() {
        let mut s = bug_spec();
        // bug with rationale rejected
        s.rationale = Some(String::from("x"));
        assert!(validate(Kind::Bug, &s).is_err());
        // feature with repro rejected
        let mut f = IssueSpec {
            title: String::from("t"),
            description: String::from("d"),
            repro: Some(String::from("r")),
            rationale: None,
        };
        assert!(validate(Kind::Feature, &f).is_err());
        // feature without repro ok
        f.repro = None;
        f.rationale = Some(String::from("because"));
        assert!(validate(Kind::Feature, &f).is_ok());
        // empty title rejected
        let bad = IssueSpec {
            title: String::from(" "),
            description: String::from("d"),
            repro: None,
            rationale: None,
        };
        assert!(validate(Kind::Bug, &bad).is_err());
    }

    #[test]
    fn file_writes_record_and_list_show_resolve() {
        let d = cfg_dir();
        let (id, path) = file(&d, Kind::Bug, &bug_spec()).expect("file");
        assert_eq!(id, "b1");
        assert!(path.ends_with("bug/b1.json"), "{path}");
        let (items, errors) = list(&d, Kind::Bug).expect("list");
        assert!(errors.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "b1");
        let rec = show(&d, Kind::Bug, "b1").expect("show");
        assert_eq!(rec.title, "crash");
        assert_eq!(rec.repro.as_deref(), Some("run x"));
        assert_eq!(rec.status, "open");
        let ts = resolve(&d, Kind::Bug, "b1").expect("resolve");
        assert!(!ts.is_empty());
        let rec2 = show(&d, Kind::Bug, "b1").expect("show");
        assert_eq!(rec2.status, "resolved");
        assert_eq!(rec2.resolved_ts.as_deref(), Some(ts.as_str()));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn show_not_found() {
        let d = cfg_dir();
        let err = show(&d, Kind::Bug, "b9").unwrap_err();
        assert!(matches!(err, CarpenterError::NotFound(_)));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn list_surfaces_corrupt_file_in_errors() {
        let d = cfg_dir();
        file(&d, Kind::Bug, &bug_spec()).expect("file");
        // inject a corrupt json file directly
        std::fs::write(d.join("bug/b2.json"), b"not-json").unwrap();
        let (items, errors) = list(&d, Kind::Bug).expect("list");
        assert_eq!(items.len(), 1);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].id.as_deref(), Some("b2"));
        assert!(errors[0].reason.starts_with("corrupt json"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
