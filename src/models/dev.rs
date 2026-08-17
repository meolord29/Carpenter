//! Dev-only data models (adr/016). Compiled solely under the `dev` feature, so
//! none of this surface reaches a release binary, the generated specs, the
//! howto, or the inlined `SKILL.md`.

use serde::Serialize;

/// One prerequisite check result for `carpenter dev check`.
#[derive(Debug, Serialize)]
pub struct DevCheckItem {
    /// check name (e.g. `"uv"`).
    pub name: String,
    /// whether the prerequisite is satisfied.
    pub ok: bool,
    /// human-readable detail (version string, or `not on PATH`).
    pub detail: String,
}
