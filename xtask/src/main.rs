//! Workspace automation tasks, invoked via `cargo xtask <task>`.

mod build;
mod bump;
mod hooks;
mod howto;
mod paths;
mod specs;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "carpenter workspace tasks", bin_name = "cargo xtask")]
struct Xtask {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Regenerate `src/howto.gen.md` from the clap surface + command examples.
    GenHowto,
    /// Regenerate the generated regions in `docs/specs/*.md` from types.
    GenSpecs,
    /// Advance the version ladder (adr/022): mutate both manifests + the
    /// version-bearing example atoms, then regenerate every derived surface.
    /// Git is left to the caller (release.yml commits and pushes the result).
    Bump {
        /// Which component to advance: `patch` (nightly merges), `minor`
        /// (promotion), `major` (reserved for critical/official changes).
        /// Mutually exclusive with `--to`.
        #[arg(value_enum)]
        part: Option<bump::Part>,
        /// Land an explicit X.Y.Z (the promotion bot: `main`'s minor+1
        /// exactly, even when the PR is reopened mid-ladder).
        #[arg(long, conflicts_with = "part")]
        to: Option<String>,
    },
    /// gen-howto + gen-specs + `cargo build` (the canonical build). `--dev`
    /// relaxes the doc/example/scenario gates and skips doc regen (the authoring
    /// loop, adr/016); `--release` builds optimized (the strict release stage).
    Build {
        /// Relax gates + skip doc regen (authoring loop). Mutually exclusive
        /// with `--release` (adr/016).
        #[arg(long)]
        dev: bool,
        /// Pass `--release` to the inner `cargo build` (used by `upgrade`).
        #[arg(long)]
        release: bool,
    },
}

fn main() -> anyhow::Result<()> {
    match Xtask::parse().cmd {
        Cmd::GenHowto => howto::run(),
        Cmd::GenSpecs => specs::run(),
        Cmd::Bump { part, to } => bump::run(part, to),
        Cmd::Build { dev, release } => build::run(dev, release),
    }
}
