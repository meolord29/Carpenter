//! `bug` commands — file/list/show/resolve (`~/.config/carpenter/bug/`).

use crate::core::bugfile::{self, Kind};
use crate::core::error::CarpenterError;
use crate::core::store::{self, Paths};
use crate::models::Data;

/// File a bug from a spec JSON (`--spec -`/file).
pub fn file(paths: &Paths, spec_json: &str) -> Result<Data, CarpenterError> {
    let spec = store::parse_spec(spec_json)?;
    let (id, path) = bugfile::file(paths.require_config_dir()?, Kind::Bug, &spec)?;
    Ok(Data::IssueFile {
        id,
        path,
        status: String::from("open"),
    })
}

/// List bugs.
pub fn list(paths: &Paths) -> Result<Data, CarpenterError> {
    let (items, errors) = bugfile::list(paths.require_config_dir()?, Kind::Bug)?;
    Ok(Data::IssueList { items, errors })
}

/// Show a bug.
pub fn show(paths: &Paths, id: &str) -> Result<Data, CarpenterError> {
    let rec = bugfile::show(paths.require_config_dir()?, Kind::Bug, id)?;
    Ok(Data::IssueShow {
        id: rec.id,
        title: rec.title,
        description: rec.description,
        repro: rec.repro,
        rationale: None,
        status: rec.status,
        resolved_ts: rec.resolved_ts,
    })
}

/// Resolve a bug.
pub fn resolve(paths: &Paths, id: &str) -> Result<Data, CarpenterError> {
    let resolved_ts = bugfile::resolve(paths.require_config_dir()?, Kind::Bug, id)?;
    Ok(Data::IssueResolve {
        id: id.into(),
        status: String::from("resolved"),
        resolved_ts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    const SPEC: &str = r#"{"title":"crash","description":"it crashes","repro":"run x"}"#;

    #[test]
    fn file_ok() {
        let paths = testutil::meta_setup();
        let data = file(&paths, SPEC).expect("file");
        let Data::IssueFile { id, status, path } = data else {
            panic!("IssueFile");
        };
        assert_eq!(id, "b1");
        assert_eq!(status, "open");
        assert!(path.ends_with("bug/b1.json"), "{path}");
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn file_rejects_rationale_as_bug() {
        let paths = testutil::meta_setup();
        let err = file(
            &paths,
            r#"{"title":"t","description":"d","rationale":"because"}"#,
        )
        .unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn file_rejects_empty_title() {
        let paths = testutil::meta_setup();
        let err = file(&paths, r#"{"title":"","description":"d"}"#).unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn list_ok_sorted() {
        let paths = testutil::meta_setup();
        file(&paths, SPEC).unwrap();
        file(&paths, r#"{"title":"b","description":"d"}"#).unwrap();
        let Data::IssueList { items, errors } = list(&paths).expect("list") else {
            panic!("IssueList");
        };
        assert!(errors.is_empty());
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "b1");
        assert_eq!(items[1].id, "b2");
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn show_ok_and_not_found() {
        let paths = testutil::meta_setup();
        file(&paths, SPEC).unwrap();
        let Data::IssueShow {
            id, repro, status, ..
        } = show(&paths, "b1").expect("show")
        else {
            panic!("IssueShow");
        };
        assert_eq!(id, "b1");
        assert_eq!(repro.as_deref(), Some("run x"));
        assert_eq!(status, "open");
        let err = show(&paths, "b9").unwrap_err();
        assert!(matches!(err, CarpenterError::NotFound(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn resolve_ok() {
        let paths = testutil::meta_setup();
        file(&paths, SPEC).unwrap();
        let Data::IssueResolve {
            status,
            resolved_ts,
            ..
        } = resolve(&paths, "b1").expect("resolve")
        else {
            panic!("IssueResolve");
        };
        assert_eq!(status, "resolved");
        assert!(!resolved_ts.is_empty());
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }
}
