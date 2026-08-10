//! Output shape for the top-level `skip` command (adr/011).

/// Representative examples for spec generation (adr/008).
pub mod examples {
    use crate::models::Data;

    /// `(cmd, input, note, data)` rows for the `skip` output contract.
    pub fn rows() -> Vec<(&'static str, &'static str, &'static str, Data)> {
        vec![
            (
                "skip --scope lesson|quiz|practice <id>",
                "id + scope",
                "`NotFound` if the id does not exist under the scope",
                Data::Skip {
                    scope: String::from("quiz"),
                    id: String::from("q1"),
                    skip: true,
                },
            ),
            (
                "skip --scope lesson|quiz|practice <id> --off",
                "id + scope",
                "clears the flag",
                Data::Skip {
                    scope: String::from("quiz"),
                    id: String::from("q1"),
                    skip: false,
                },
            ),
        ]
    }
}
