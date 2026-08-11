//! `feature` commands — file/list/show/resolve (`~/.config/carpenter/feature_request/`).

use crate::core::bugfile::{self, Kind};
use crate::core::error::CarpenterError;
use crate::core::store::{self, Paths};
use crate::models::Data;

/// File a feature request from a spec YAML (`--spec -`/file).
pub fn file(paths: &Paths, spec_text: &str) -> Result<Data, CarpenterError> {
    let spec = store::parse_spec(spec_text)?;
    let (id, path) = bugfile::file(paths.require_config_dir()?, Kind::Feature, &spec)?;
    Ok(Data::IssueFile {
        id,
        path,
        status: String::from("open"),
    })
}

/// List feature requests.
pub fn list(paths: &Paths) -> Result<Data, CarpenterError> {
    let (items, errors) = bugfile::list(paths.require_config_dir()?, Kind::Feature)?;
    Ok(Data::IssueList { items, errors })
}

/// Show a feature request.
pub fn show(paths: &Paths, id: &str) -> Result<Data, CarpenterError> {
    let rec = bugfile::show(paths.require_config_dir()?, Kind::Feature, id)?;
    Ok(Data::IssueShow {
        id: rec.id,
        title: rec.title,
        description: rec.description,
        repro: None,
        rationale: rec.rationale,
        status: rec.status,
        resolved_ts: rec.resolved_ts,
    })
}

/// Resolve a feature request.
pub fn resolve(paths: &Paths, id: &str) -> Result<Data, CarpenterError> {
    let resolved_ts = bugfile::resolve(paths.require_config_dir()?, Kind::Feature, id)?;
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

    const SPEC: &str =
        "title: add dark mode\ndescription: theme switch\nrationale: users ask for it\n";

    #[test]
    fn file_ok() {
        let paths = testutil::meta_setup();
        let Data::IssueFile { id, status, path } = file(&paths, SPEC).expect("file") else {
            panic!("IssueFile");
        };
        assert_eq!(id, "f1");
        assert_eq!(status, "open");
        assert!(path.ends_with("feature_request/f1.json"), "{path}");
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn file_rejects_repro_as_feature() {
        let paths = testutil::meta_setup();
        let err = file(&paths, "title: t\ndescription: d\nrepro: run x\n").unwrap_err();
        assert!(matches!(err, CarpenterError::ValidationError(_)));
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn list_ok() {
        let paths = testutil::meta_setup();
        file(&paths, SPEC).unwrap();
        let Data::IssueList { items, errors } = list(&paths).expect("list") else {
            panic!("IssueList");
        };
        assert!(errors.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "f1");
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }

    #[test]
    fn show_ok_and_not_found() {
        let paths = testutil::meta_setup();
        file(&paths, SPEC).unwrap();
        let Data::IssueShow {
            rationale, status, ..
        } = show(&paths, "f1").expect("show")
        else {
            panic!("IssueShow");
        };
        assert_eq!(rationale.as_deref(), Some("users ask for it"));
        assert_eq!(status, "open");
        let err = show(&paths, "f9").unwrap_err();
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
        } = resolve(&paths, "f1").expect("resolve")
        else {
            panic!("IssueResolve");
        };
        assert_eq!(status, "resolved");
        assert!(!resolved_ts.is_empty());
        let _ = std::fs::remove_dir_all(paths.root);
        let _ = std::fs::remove_dir_all(paths.config_dir.unwrap());
    }
}
