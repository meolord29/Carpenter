//! build: the canonical build. Two stages (adr/016):
//! - `--dev`: `cargo build --features dev` only (gates relaxed, no doc regen) —
//!   the authoring loop.
//! - (default)/`--release`: gen-howto + gen-specs + `cargo build [--release]`
//!   (strict gates enforced by `build.rs`).

use std::process::Command;

/// Run the canonical build. `dev` and `release` are mutually exclusive: `dev`
/// relaxes the gates (authoring), `release` enforces them (ship). Neither flag
/// = the strict debug build + doc regen (the everyday verification).
pub fn run(dev: bool, release: bool) -> anyhow::Result<()> {
    if dev && release {
        anyhow::bail!(
            "--dev and --release are mutually exclusive (--dev relaxes gates; --release enforces them) — see adr/016"
        );
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));

    if dev {
        // Dev stage: relaxed gates. Skip gen-howto/gen-specs — atoms may be
        // mid-authoring, and regen must always run against the strict (non-dev)
        // view of `app::cli()` so dev-only surface never enters the manual.
        let status = Command::new(&cargo)
            .args(["build", "--features", "dev"])
            .status()?;
        if !status.success() {
            anyhow::bail!("cargo build --features dev failed");
        }
        return Ok(());
    }

    // Strict stage: regenerate surfaces (xtask links carpenter WITHOUT `dev`,
    // so the generated docs exclude dev-only surface), then the gated build.
    crate::howto::run()?;
    crate::specs::run()?;
    let mut cmd = Command::new(&cargo);
    cmd.arg("build");
    if release {
        cmd.arg("--release");
    }
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("cargo build failed");
    }
    Ok(())
}
