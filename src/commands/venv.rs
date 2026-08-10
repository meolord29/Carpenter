//! `venv` commands — uv-managed course venv (create/sync/list/add).

use std::process::Output;

use crate::core::error::CarpenterError;
use crate::core::exec;
use crate::core::store::{self, Paths};
use crate::models::venv::Package;
use crate::models::Data;

/// The canonical base deps installed into every course venv.
pub const BASE_DEPS: &[&str] = &["jupyterlab", "nbconvert", "nbclient", "ipykernel"];

fn require_venv(paths: &Paths, course: &str) -> Result<(), CarpenterError> {
    if !paths.course(course).join(".venv").exists() {
        return Err(CarpenterError::StoreError(format!(
            "no course venv for {course} — run `carpenter venv create` first"
        )));
    }
    Ok(())
}

fn packages_from(output: &Output) -> Vec<Package> {
    parse_packages(&String::from_utf8_lossy(&output.stdout))
}

/// Parse `uv pip list` stdout into packages (skips the header/dashed separator).
pub fn parse_packages(text: &str) -> Vec<Package> {
    let mut pkgs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('-') || line.contains("Version") {
            continue;
        }
        let mut it = line.split_whitespace();
        if let (Some(name), Some(version)) = (it.next(), it.next()) {
            pkgs.push(Package {
                name: name.into(),
                version: version.into(),
            });
        }
    }
    pkgs
}

/// Create the course venv (requires `uv` on PATH).
pub fn create(
    paths: &Paths,
    course_slug: &str,
    python: Option<&str>,
) -> Result<Data, CarpenterError> {
    exec::require_uv(exec::uv_available())?;
    let course_dir = paths.course(course_slug);
    let venv = course_dir.join(".venv");
    if venv.exists() {
        return Err(CarpenterError::AlreadyExists(format!(
            ".venv for course {course_slug} (re-run `carpenter venv sync` to update)"
        )));
    }
    let mut venv_args = vec!["venv"];
    if let Some(p) = python {
        venv_args.push("--python");
        venv_args.push(p);
    }
    exec::run_uv_or_store(&venv_args, &course_dir)?;
    let deps_lit = BASE_DEPS
        .iter()
        .map(|d| format!("\"{d}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let pyproject = format!(
        "[project]\nname = \"{course_slug}\"\nversion = \"0\"\nrequires-python = \">=3.8\"\ndependencies = [{deps_lit}]\n"
    );
    store::atomic_write(&course_dir.join("pyproject.toml"), pyproject.as_bytes())?;
    exec::run_uv_or_store(&["sync"], &course_dir)?;
    Ok(Data::VenvCreate {
        course: course_slug.into(),
        python: python.unwrap_or("default").into(),
        path: venv.display().to_string(),
        deps: BASE_DEPS.iter().map(|s| s.to_string()).collect(),
    })
}

/// Sync the venv (`uv sync`).
pub fn sync(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    require_venv(paths, course_slug)?;
    exec::run_uv_or_store(&["sync"], &paths.course(course_slug))?;
    Ok(Data::VenvSync {
        course: course_slug.into(),
        synced: true,
    })
}

/// List installed packages (`uv pip list`).
pub fn list(paths: &Paths, course_slug: &str) -> Result<Data, CarpenterError> {
    require_venv(paths, course_slug)?;
    let out = exec::run_uv_or_store(&["pip", "list"], &paths.course(course_slug))?;
    Ok(Data::VenvList {
        course: course_slug.into(),
        packages: packages_from(&out),
    })
}

/// Add packages (`uv add`).
pub fn add(paths: &Paths, course_slug: &str, pkgs: &[String]) -> Result<Data, CarpenterError> {
    require_venv(paths, course_slug)?;
    let mut args: Vec<&str> = vec!["add"];
    for p in pkgs {
        args.push(p.as_str());
    }
    exec::run_uv_or_store(&args, &paths.course(course_slug))?;
    let out = exec::run_uv_or_store(&["pip", "list"], &paths.course(course_slug))?;
    Ok(Data::VenvAdd {
        course: course_slug.into(),
        added: pkgs.to_vec(),
        packages: packages_from(&out),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_requires_uv_and_rejects_existing_venv() {
        let (paths, slug) = crate::commands::testutil::setup();
        // pre-create a .venv dir → AlreadyExists (no uv invocation)
        std::fs::create_dir_all(paths.course(&slug).join(".venv")).unwrap();
        let err = create(&paths, &slug, None).unwrap_err();
        assert!(matches!(err, CarpenterError::AlreadyExists(_)), "{err:?}");
    }

    #[test]
    fn sync_requires_venv() {
        let (paths, slug) = crate::commands::testutil::setup();
        let err = sync(&paths, &slug).unwrap_err();
        assert!(matches!(err, CarpenterError::StoreError(_)));
    }

    #[test]
    fn list_requires_venv() {
        let (paths, slug) = crate::commands::testutil::setup();
        let err = list(&paths, &slug).unwrap_err();
        assert!(matches!(err, CarpenterError::StoreError(_)));
    }

    #[test]
    fn add_requires_venv() {
        let (paths, slug) = crate::commands::testutil::setup();
        let err = add(&paths, &slug, &["numpy".into()]).unwrap_err();
        assert!(matches!(err, CarpenterError::StoreError(_)));
    }

    #[test]
    fn parse_packages_skips_header() {
        let pkgs = parse_packages(
            "Package          Version\n---------------  -------\nnbconvert        7.16.4\njupyterlab       4.2.4\n",
        );
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "nbconvert");
        assert_eq!(pkgs[0].version, "7.16.4");
    }
}
