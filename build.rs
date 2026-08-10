//! Compile-time gate (adr/007): every command fn in `src/commands/` is
//! self-documenting and tested. A miss aborts the build — no binary is produced.
//!
//! A "command" is any `pub fn` in `src/commands/` whose return type ends in
//! `Result`. Helpers must live in `core/`, never in `commands/`.
//!
//! For each command fn `<module>::<name>` we require:
//! - a worked-example file at `docs/examples/<module>/<name>.md` (the single
//!   source scraped into `howto` by `xtask gen-howto`); AND
//! - a paired `#[test] fn <name>_*` in the same module.
//!
//! The example file (not an inline `///` fence) is the atom — see the Update
//! block in adr/007.

use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/commands");
    println!("cargo:rerun-if-changed=docs/examples");
    let dir = Path::new("src/commands");
    let examples = Path::new("docs/examples");
    if !dir.exists() {
        return;
    }
    let mut errors: Vec<String> = Vec::new();
    for file in walk_rs(dir) {
        let Ok(src) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(parsed) = syn::parse_file(&src) else {
            continue;
        };
        let (cmds, tests) = scan(&parsed.items);
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        for name in cmds {
            let prefix = format!("{name}_");
            let has_test = tests.iter().any(|t| t.starts_with(&prefix));
            let example = examples.join(&stem).join(format!("{name}.md"));
            if !example.exists() {
                errors.push(format!(
                    "commands/{stem}.rs:{name}: missing example file {}",
                    example.display()
                ));
            }
            if !has_test {
                errors.push(format!(
                    "commands/{stem}.rs:{name}: missing #[test] fn {name}_*"
                ));
            }
        }
    }
    if errors.is_empty() {
        return;
    }
    for e in &errors {
        eprintln!("{e}");
    }
    std::process::exit(1);
}

/// Recursively collect `command_name`s and `test_name`s from items.
fn scan(items: &[syn::Item]) -> (Vec<String>, Vec<String>) {
    let mut cmds = Vec::new();
    let mut tests = Vec::new();
    for item in items {
        match item {
            syn::Item::Fn(f) => {
                let name = f.sig.ident.to_string();
                if f.attrs.iter().any(|a| a.path().is_ident("test")) {
                    tests.push(name.clone());
                }
                if is_public(f) && returns_result(&f.sig) {
                    cmds.push(name);
                }
            }
            syn::Item::Mod(m) => {
                if let Some((_, nested)) = &m.content {
                    let (c, t) = scan(nested);
                    cmds.extend(c);
                    tests.extend(t);
                }
            }
            _ => {}
        }
    }
    (cmds, tests)
}

fn is_public(f: &syn::ItemFn) -> bool {
    matches!(f.vis, syn::Visibility::Public(_))
}

fn returns_result(sig: &syn::Signature) -> bool {
    let syn::ReturnType::Type(_, ty) = &sig.output else {
        return false;
    };
    matches!(peel(ty), syn::Type::Path(tp) if tp.path.segments.last().map(|s| s.ident == "Result").unwrap_or(false))
}

fn peel(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Paren(p) = ty {
        ty = &p.elem;
    }
    ty
}

fn walk_rs(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk_rs(&p));
        } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
            out.push(p);
        }
    }
    out
}
