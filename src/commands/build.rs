//! `build` — scaffold a course at an arbitrary path: creates `course.json`,
//! `course.db`, and `lessons/`. Unlike `course create` (which lives under
//! `<root>/courses/`), `build` places a course wherever the caller asks.

use std::path::Path;

use crate::core::error::CarpenterError;
use crate::core::store::{self, Paths};
use crate::models::Data;

/// `build <path>`.
pub fn build(_paths: &Paths, target: &str) -> Result<Data, CarpenterError> {
    let dir = Path::new(target);
    let basename = dir.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
        CarpenterError::ValidationError(format!("cannot derive slug from {target:?}"))
    })?;
    let slug = store::slugify(basename)?;
    if dir.exists() {
        return Err(CarpenterError::AlreadyExists(format!(
            "path {target} already exists"
        )));
    }
    store::init_course_dir(dir, &slug, &slug, "", "")?;
    std::fs::create_dir_all(dir.join("lessons")).map_err(store::io_to_store)?;
    Ok(Data::Build {
        path: dir.display().to_string(),
        slug,
        created: vec![
            String::from("course.json"),
            String::from("course.db"),
            String::from("lessons/"),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::testutil;

    #[test]
    fn build_scaffolds_three_artifacts() {
        let paths = testutil::meta_setup();
        let target = paths.root.join("elsewhere/ds");
        let Data::Build {
            slug,
            created,
            path,
        } = build(&paths, target.to_str().unwrap()).expect("build")
        else {
            panic!("Build");
        };
        assert_eq!(slug, "ds");
        assert_eq!(created, vec!["course.json", "course.db", "lessons/"]);
        assert!(Path::new(&path).join("course.json").exists());
        assert!(Path::new(&path).join("course.db").exists());
        assert!(Path::new(&path).join("lessons").is_dir());
        let _ = std::fs::remove_dir_all(&paths.root);
    }

    #[test]
    fn build_rejects_existing_path() {
        let paths = testutil::meta_setup();
        let target = paths.root.join("ds");
        std::fs::create_dir_all(&target).unwrap();
        let err = build(&paths, target.to_str().unwrap()).unwrap_err();
        assert!(matches!(err, CarpenterError::AlreadyExists(_)));
        let _ = std::fs::remove_dir_all(&paths.root);
    }
}
