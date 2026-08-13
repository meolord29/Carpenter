//! gen-specs: regenerate the `<!-- BEGIN/END GENERATED -->` regions in
//! `docs/specs/*.md` from the registered `*Spec`/`Data` examples.
//!
//! Only files with a registry entry are touched; all others are left as-is
//! (hand-maintained until their types land). Prose outside the markers is always
//! preserved.

use carpenter::models::examples;

/// Rewrite the generated regions for every registered spec entry.
pub fn run() -> anyhow::Result<()> {
    let specs_dir = crate::paths::workspace_root().join("docs/specs");
    for entry in examples::all() {
        let path = specs_dir.join(entry.file);
        let Ok(original) = std::fs::read_to_string(&path) else {
            continue; // spec file not present yet — skip
        };
        let updated = replace_region(&original, &entry.markdown);
        if updated != original {
            std::fs::write(&path, &updated)?;
            println!("gen-specs: updated {}", entry.file);
        }
    }
    Ok(())
}

/// Replace the content between the markers, preserving everything outside.
pub fn replace_region(src: &str, replacement: &str) -> String {
    const BEGIN: &str = "<!-- BEGIN GENERATED -->";
    const END: &str = "<!-- END GENERATED -->";
    let Some(begin_idx) = src.find(BEGIN) else {
        return src.to_owned();
    };
    let Some(end_rel) = src[begin_idx..].find(END) else {
        return src.to_owned();
    };
    let end_idx = begin_idx + end_rel;
    let mut out = String::with_capacity(src.len() + replacement.len());
    out.push_str(&src[..begin_idx]);
    out.push_str(BEGIN);
    out.push('\n');
    out.push_str(replacement);
    out.push('\n');
    out.push_str(&src[end_idx..]);
    out
}

#[cfg(all(test, not(feature = "dev")))]
#[test]
fn specs_marker_regions_are_fresh() {
    let specs_dir = crate::paths::workspace_root().join("docs/specs");
    for entry in examples::all() {
        let path = specs_dir.join(entry.file);
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("spec file {} should exist", path.display()));
        let regenerated = replace_region(&committed, &entry.markdown);
        assert_eq!(
            regenerated, committed,
            "docs/specs/{} is stale; run `cargo xtask gen-specs`",
            entry.file
        );
    }
}
