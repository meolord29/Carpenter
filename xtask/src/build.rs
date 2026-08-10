//! build: gen-howto + gen-specs + `cargo build` (the canonical build).

use std::process::Command;

/// Run the full canonical build. `release` passes `--release` to the inner build.
pub fn run(release: bool) -> anyhow::Result<()> {
    crate::howto::run()?;
    crate::specs::run()?;
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let mut cmd = Command::new(cargo);
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
