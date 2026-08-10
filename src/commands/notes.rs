//! `notes` commands — add/show/list/update/resolve/remove.
//!
//! `related_open` is an advisory hint (open notes sharing ≥1 tag, excluding
//! self); `recurrence` is authored and never auto-changed.

use crate::core::error::CarpenterError;
use crate::core::store;
use crate::core::time;
use crate::core::{db, store::Paths};
use crate::models::common::RowError;
use crate::models::note::{NoteItem, NoteSpec};
use crate::models::Data;

/// Allowed note kinds (`docs/specs/06-note-spec.md`).
const KINDS: [&str; 5] = ["gap", "mistake", "strength", "pattern", "progress"];
/// Allowed recurrence values.
const RECURRENCES: [&str; 2] = ["new", "recurring"];

fn validate_note_spec(spec: &NoteSpec) -> Result<(), CarpenterError> {
    if !KINDS.contains(&spec.kind.as_str()) {
        return Err(CarpenterError::ValidationError(format!(
            "kind must be one of {} (got {:?})",
            KINDS.join("|"),
            spec.kind
        )));
    }
    if !RECURRENCES.contains(&spec.recurrence.as_str()) {
        return Err(CarpenterError::ValidationError(format!(
            "recurrence must be new|recurring (got {:?})",
            spec.recurrence
        )));
    }
    if spec.text.trim().is_empty() {
        return Err(CarpenterError::ValidationError(
            "text must be non-empty".into(),
        ));
    }
    Ok(())
}

/// Build a [`NoteItem`] from a row; `Err(reason)` if `tags` JSON is corrupt.
fn note_item(row: db::NoteDb) -> Result<NoteItem, String> {
    let tags: Vec<String> = serde_json::from_str(&row.tags)
        .map_err(|e| format!("corrupt tags json for {}: {e}", row.id))?;
    Ok(NoteItem {
        id: row.id,
        kind: row.kind,
        tags,
        status: row.status,
        recurrence: row.recurrence,
        related: row.related,
        text: row.text,
    })
}

/// Open notes (excluding `self_id`) sharing ≥1 tag with `tags`.
fn related_open(
    conn: &rusqlite::Connection,
    self_id: &str,
    tags: &[String],
) -> Result<Vec<String>, CarpenterError> {
    let mut out = Vec::new();
    for row in db::list_notes(conn)? {
        if row.status != "open" || row.id == self_id {
            continue;
        }
        let Ok(other_tags): Result<Vec<String>, _> = serde_json::from_str(&row.tags) else {
            continue;
        };
        if tags.iter().any(|t| other_tags.contains(t)) {
            out.push(row.id);
        }
    }
    Ok(out)
}

/// Add a note from a spec; echoes the new row + an advisory `related_open`.
pub fn add(paths: &Paths, course_slug: &str, spec_json: &str) -> Result<Data, CarpenterError> {
    let spec: NoteSpec = store::parse_spec(spec_json)?;
    validate_note_spec(&spec)?;
    let conn = db::open_course(paths, course_slug)?;
    let id = db::next_id(&conn, "notes", "n")?;
    let now = time::now_iso();
    let related = spec.related.clone().unwrap_or_default();
    let tags_json = serde_json::to_string(&spec.tags)
        .map_err(|e| CarpenterError::StoreError(format!("tags encode failed: {e}")))?;
    let row = db::NoteDb {
        id: id.clone(),
        ts: now.clone(),
        updated_ts: now,
        kind: spec.kind.clone(),
        tags: tags_json,
        status: String::from("open"),
        recurrence: spec.recurrence.clone(),
        related: related.clone(),
        text: spec.text.clone(),
    };
    db::insert_note(&conn, &row)?;
    let related_open = related_open(&conn, &id, &spec.tags)?;
    Ok(Data::NotesAdd {
        id,
        kind: spec.kind,
        tags: spec.tags,
        status: String::from("open"),
        recurrence: spec.recurrence,
        related,
        text: spec.text,
        related_open,
    })
}

/// Show a note (`{notes:[<row>]}`).
pub fn show(paths: &Paths, course_slug: &str, id: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let row = db::get_note(&conn, id)?;
    let item = note_item(row).map_err(CarpenterError::StoreError)?;
    Ok(Data::NotesShow { notes: vec![item] })
}

/// List notes; corrupt rows surface in `errors[]`.
pub fn list(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let mut notes = Vec::new();
    let mut errors = Vec::new();
    for row in db::list_notes(&conn)? {
        let id = row.id.clone();
        match note_item(row) {
            Ok(item) => notes.push(item),
            Err(reason) => errors.push(RowError {
                id: Some(id),
                reason,
            }),
        }
    }
    Ok(Data::NotesList { notes, errors })
}

/// Replace a note's authored fields from a spec (status is preserved).
pub fn update(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    spec_json: &str,
) -> Result<Data, CarpenterError> {
    let spec: NoteSpec = store::parse_spec(spec_json)?;
    validate_note_spec(&spec)?;
    let conn = db::open_course(paths, course_slug)?;
    let existing = db::get_note(&conn, id)?;
    let now = time::now_iso();
    let related = spec.related.clone().unwrap_or_default();
    let tags_json = serde_json::to_string(&spec.tags)
        .map_err(|e| CarpenterError::StoreError(format!("tags encode failed: {e}")))?;
    db::update_note(
        &conn,
        id,
        &spec.kind,
        &tags_json,
        &spec.recurrence,
        &related,
        &spec.text,
        &now,
    )?;
    let updated = NoteItem {
        id: id.into(),
        kind: spec.kind,
        tags: spec.tags,
        status: existing.status,
        recurrence: spec.recurrence,
        related,
        text: spec.text,
    };
    Ok(Data::NotesUpdate {
        id: id.into(),
        updated,
    })
}

/// Resolve a note (sets `status='resolved'`).
pub fn resolve(paths: &Paths, course_slug: &str, id: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let _ = db::get_note(&conn, id)?;
    let now = time::now_iso();
    db::set_note_status(&conn, id, "resolved", &now)?;
    Ok(Data::NotesResolve {
        id: id.into(),
        status: String::from("resolved"),
    })
}

/// Remove a note (`--force` required).
pub fn remove(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    force: bool,
) -> Result<Data, CarpenterError> {
    if !force {
        return Err(CarpenterError::Conflict(format!(
            "remove requires --force: note {id}"
        )));
    }
    let conn = db::open_course(paths, course_slug)?;
    let _ = db::get_note(&conn, id)?;
    db::delete_note(&conn, id)?;
    Ok(Data::NotesRemove {
        id: id.into(),
        deleted: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    const SPEC: &str = r#"{"kind":"gap","tags":["recursion"],"text":"struggles with base cases"}"#;

    fn note_id(data: &Data) -> String {
        match data {
            Data::NotesAdd { id, .. } => id.clone(),
            _ => panic!("not NotesAdd"),
        }
    }

    #[test]
    fn add_ok() {
        let (paths, slug) = testutil::setup();
        let data = add(&paths, &slug, SPEC).expect("add");
        match data {
            Data::NotesAdd {
                id,
                status,
                related_open,
                kind,
                ..
            } => {
                assert!(id.starts_with('n'), "{id}");
                assert_eq!(status, "open");
                assert_eq!(kind, "gap");
                assert!(related_open.is_empty());
            }
            _ => panic!("NotesAdd"),
        }
    }

    #[test]
    fn add_rejects_bad_kind() {
        let (paths, slug) = testutil::setup();
        let err = add(&paths, &slug, r#"{"kind":"nope","text":"t"}"#).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
    }

    #[test]
    fn add_rejects_empty_text() {
        let (paths, slug) = testutil::setup();
        let err = add(&paths, &slug, r#"{"kind":"gap","text":"  "}"#).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
    }

    #[test]
    fn related_open_excludes_self_and_overlaps_tags() {
        let (paths, slug) = testutil::setup();
        // pre-existing open note with a shared tag
        add(
            &paths,
            &slug,
            r#"{"kind":"gap","tags":["recursion","lists"],"text":"a"}"#,
        )
        .unwrap();
        // pre-existing resolved note with a shared tag (must be excluded)
        let resolved = note_id(
            &add(
                &paths,
                &slug,
                r#"{"kind":"gap","tags":["recursion"],"text":"b"}"#,
            )
            .unwrap(),
        );
        resolve(&paths, &slug, &resolved).unwrap();
        // pre-existing open note with no overlap (must be excluded)
        add(
            &paths,
            &slug,
            r#"{"kind":"strength","tags":["loops"],"text":"c"}"#,
        )
        .unwrap();

        let data = add(
            &paths,
            &slug,
            r#"{"kind":"mistake","tags":["lists"],"text":"new note"}"#,
        )
        .expect("add");
        match data {
            Data::NotesAdd {
                related_open, id, ..
            } => {
                // only the first note shares a tag ("lists") and is open; self excluded.
                assert_ne!(id, related_open[0], "self must be excluded");
                assert_eq!(related_open.len(), 1, "{related_open:?}");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn show_ok() {
        let (paths, slug) = testutil::setup();
        let id = note_id(&add(&paths, &slug, SPEC).unwrap());
        let Data::NotesShow { notes } = show(&paths, &slug, &id).expect("show") else {
            panic!();
        };
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, id);
    }

    #[test]
    fn show_not_found() {
        let (paths, slug) = testutil::setup();
        let err = show(&paths, &slug, "n99").unwrap_err();
        assert!(matches!(err, CarpenterError::NotFound(_)));
    }

    #[test]
    fn list_surfaces_corrupt_rows_in_errors() {
        let (paths, slug) = testutil::setup();
        add(&paths, &slug, SPEC).unwrap();
        // inject a row with corrupt tags json directly (the helper never writes bad json)
        let conn = db::open_course(&paths, &slug).unwrap();
        conn.execute(
            "INSERT INTO notes (id,ts,updated_ts,kind,tags,status,recurrence,related,text) \
             VALUES ('n9','t','t','gap','not-json','open','new','','x')",
            [],
        )
        .unwrap();
        drop(conn);
        let Data::NotesList { notes, errors } = list(&paths, &slug).expect("list") else {
            panic!();
        };
        assert_eq!(notes.len(), 1);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].id.as_deref(), Some("n9"));
        assert!(
            errors[0].reason.starts_with("corrupt tags json for n9"),
            "{}",
            errors[0].reason
        );
    }

    #[test]
    fn update_replaces_fields_and_preserves_status() {
        let (paths, slug) = testutil::setup();
        let id = note_id(&add(&paths, &slug, SPEC).unwrap());
        resolve(&paths, &slug, &id).unwrap();
        let Data::NotesUpdate { updated, .. } = update(
            &paths,
            &slug,
            &id,
            r#"{"kind":"pattern","tags":["x"],"recurrence":"recurring","text":"edited"}"#,
        )
        .expect("update") else {
            panic!();
        };
        assert_eq!(updated.kind, "pattern");
        assert_eq!(updated.recurrence, "recurring");
        assert_eq!(updated.status, "resolved"); // preserved across update
    }

    #[test]
    fn resolve_ok() {
        let (paths, slug) = testutil::setup();
        let id = note_id(&add(&paths, &slug, SPEC).unwrap());
        let Data::NotesResolve { status, .. } = resolve(&paths, &slug, &id).expect("resolve")
        else {
            panic!();
        };
        assert_eq!(status, "resolved");
    }

    #[test]
    fn remove_requires_force() {
        let (paths, slug) = testutil::setup();
        let id = note_id(&add(&paths, &slug, SPEC).unwrap());
        let err = remove(&paths, &slug, &id, false).unwrap_err();
        assert!(matches!(err, CarpenterError::Conflict(_)));
        let Data::NotesRemove { deleted, .. } = remove(&paths, &slug, &id, true).expect("remove")
        else {
            panic!();
        };
        assert!(deleted);
        assert!(matches!(
            show(&paths, &slug, &id).unwrap_err(),
            CarpenterError::NotFound(_)
        ));
    }
}
