//! `goal` commands — add/list/update/remove.

use crate::core::error::CarpenterError;
use crate::core::store;
use crate::core::time;
use crate::core::{db, status, store::Paths};
use crate::models::goal::{GoalListItem, GoalRow, GoalSpec};
use crate::models::Data;

fn validate_covered_by(
    conn: &rusqlite::Connection,
    covered_by: &[String],
) -> Result<(), CarpenterError> {
    for lid in covered_by {
        if !db::lesson_exists(conn, lid)? {
            return Err(CarpenterError::ValidationError(format!(
                "unresolvable lesson id: {lid}"
            )));
        }
    }
    Ok(())
}

/// Add a goal from a spec.
pub fn add(paths: &Paths, course_slug: &str, spec_text: &str) -> Result<Data, CarpenterError> {
    let spec: GoalSpec = store::parse_spec(spec_text)?;
    if spec.text.trim().is_empty() {
        return Err(CarpenterError::ValidationError(
            "text must be non-empty".into(),
        ));
    }
    let conn = db::open_course(paths, course_slug)?;
    validate_covered_by(&conn, &spec.covered_by)?;
    let id = db::next_id(&conn, "goals", "g")?;
    let row = GoalRow {
        id: id.clone(),
        scope: String::from("course"),
        scope_id: course_slug.into(),
        text: spec.text.clone(),
        status: String::from("pending"),
        covered_by: spec.covered_by.clone(),
        override_flag: false,
        created_at: time::now_iso(),
    };
    db::insert_goal(&conn, &row)?;
    Ok(Data::GoalAdd {
        id,
        text: spec.text,
        covered_by: spec.covered_by,
        status: String::from("pending"),
    })
}

/// List goals with effective + derived statuses.
pub fn list(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let rows = db::list_goals(&conn)?;
    let mut goals = Vec::with_capacity(rows.len());
    for g in &rows {
        let effective = status::goal_effective(&conn, g)?;
        let derived = status::goal_derived(&conn, g)?;
        goals.push(GoalListItem {
            id: g.id.clone(),
            text: g.text.clone(),
            status: effective.as_str().into(),
            derived_status: derived.as_str().into(),
            covered_by: g.covered_by.clone(),
        });
    }
    Ok(Data::GoalList { goals })
}

/// Update a goal's status (pin via `pending|achieved|skipped`, or `derived` to
/// resume) and/or rewrite `covered_by`.
pub fn update(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    status_arg: Option<&str>,
    covered_by: Option<&[String]>,
) -> Result<Data, CarpenterError> {
    let conn = db::open_course(paths, course_slug)?;
    let row = db::get_goal(&conn, id)?;
    let mut new_override = row.override_flag;
    let mut pinned_status = row.status.clone();
    if let Some(s) = status_arg {
        match s {
            "pending" | "achieved" | "skipped" => {
                new_override = true;
                pinned_status = s.into();
            }
            "derived" => new_override = false,
            other => {
                return Err(CarpenterError::ValidationError(format!(
                    "invalid --status {other:?} (pending|achieved|skipped|derived)"
                )))
            }
        }
    }
    let new_covered = match covered_by {
        Some(cb) => {
            validate_covered_by(&conn, cb)?;
            cb.to_vec()
        }
        None => row.covered_by.clone(),
    };
    let effective = if new_override {
        pinned_status.clone()
    } else {
        status::goal_derived_from(&conn, &new_covered)?
            .as_str()
            .into()
    };
    db::update_goal(&conn, id, &effective, new_override, &new_covered)?;
    Ok(Data::GoalUpdate {
        id: id.into(),
        status: effective,
        override_field: new_override,
        covered_by: new_covered,
    })
}

/// Remove a goal (`--force` required).
pub fn remove(
    paths: &Paths,
    course_slug: &str,
    id: &str,
    force: bool,
) -> Result<Data, CarpenterError> {
    if !force {
        return Err(CarpenterError::Conflict(format!(
            "remove requires --force: goal {id}"
        )));
    }
    let conn = db::open_course(paths, course_slug)?;
    let _ = db::get_goal(&conn, id)?;
    db::delete_goal(&conn, id)?;
    Ok(Data::GoalRemove {
        id: id.into(),
        deleted: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;
    use crate::models::Data;

    const SPEC: &str = "text: learn hashing\ncovered_by: []\n";

    fn goal_id(data: &Data) -> String {
        match data {
            Data::GoalAdd { id, .. } => id.clone(),
            _ => panic!("not GoalAdd"),
        }
    }

    #[test]
    fn add_ok() {
        let (paths, slug) = testutil::setup();
        let data = add(&paths, &slug, SPEC).expect("add");
        match data {
            Data::GoalAdd { status, .. } => assert_eq!(status, "pending"),
            _ => panic!("GoalAdd"),
        }
    }

    #[test]
    fn add_rejects_empty_text() {
        let (paths, slug) = testutil::setup();
        let err = add(&paths, &slug, "text: '  '\ncovered_by: []\n").unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
    }

    #[test]
    fn add_rejects_unresolved_covered_by() {
        let (paths, slug) = testutil::setup();
        let err = add(&paths, &slug, "text: t\ncovered_by:\n  - nope-lesson\n").unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
    }

    #[test]
    fn list_ok() {
        let (paths, slug) = testutil::setup();
        add(&paths, &slug, SPEC).unwrap();
        let data = list(&paths, &slug).expect("list");
        match data {
            Data::GoalList { goals } => {
                assert_eq!(goals.len(), 1);
                assert_eq!(goals[0].derived_status, "pending");
            }
            _ => panic!("GoalList"),
        }
    }

    #[test]
    fn update_pins_status() {
        let (paths, slug) = testutil::setup();
        let id = goal_id(&add(&paths, &slug, SPEC).unwrap());
        let data = update(&paths, &slug, &id, Some("achieved"), None).expect("update");
        match data {
            Data::GoalUpdate {
                status,
                override_field,
                ..
            } => {
                assert_eq!(status, "achieved");
                assert!(override_field);
            }
            _ => panic!("GoalUpdate"),
        }
    }

    #[test]
    fn update_derived_clears_override() {
        let (paths, slug) = testutil::setup();
        let id = goal_id(&add(&paths, &slug, SPEC).unwrap());
        update(&paths, &slug, &id, Some("achieved"), None).unwrap();
        let data = update(&paths, &slug, &id, Some("derived"), None).expect("update");
        match data {
            Data::GoalUpdate {
                override_field,
                status,
                ..
            } => {
                assert!(!override_field);
                assert_eq!(status, "pending"); // empty covered_by ⇒ pending
            }
            _ => panic!("GoalUpdate"),
        }
    }

    #[test]
    fn remove_ok() {
        let (paths, slug) = testutil::setup();
        let id = goal_id(&add(&paths, &slug, SPEC).unwrap());
        assert!(matches!(
            remove(&paths, &slug, &id, true).expect("remove"),
            Data::GoalRemove { deleted: true, .. }
        ));
    }

    #[test]
    fn remove_without_force_conflicts() {
        let (paths, slug) = testutil::setup();
        let id = goal_id(&add(&paths, &slug, SPEC).unwrap());
        let err = remove(&paths, &slug, &id, false).unwrap_err();
        assert!(matches!(err, CarpenterError::Conflict(_)));
    }
}
