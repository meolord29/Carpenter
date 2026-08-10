//! `howto` — print the build-time-generated command manual.

use crate::core::error::CarpenterError;
use crate::manual;
use crate::models::Data;

/// Print the build-time-generated command manual.
pub fn howto() -> Result<Data, CarpenterError> {
    Ok(Data::Howto {
        howto: manual::MANUAL.to_string(),
    })
}

#[cfg(test)]
#[test]
fn howto_returns_nonempty_manual() {
    if let Data::Howto { howto } = howto().expect("howto succeeds") {
        assert!(!howto.is_empty());
    } else {
        panic!("expected Howto variant");
    }
}
