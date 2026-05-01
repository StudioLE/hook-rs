//! Individual parsed shell command.

use crate::prelude::*;

/// A simple command.
///
/// Example: `git diff --stat HEAD~3`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimpleContext {
    /// Utility name.
    ///
    /// Examples: `git`, `head`
    pub name: String,
    /// Positional arguments and flags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Whether the command has a heredoc redirect.
    #[serde(skip_serializing_if = "is_false")]
    pub has_heredoc: bool,
    /// Whether any argument contains a command substitution.
    #[serde(skip_serializing_if = "is_false")]
    pub contains_substitution: bool,
    /// Environment variable assignments prefixed to the command.
    ///
    /// Example: `RUST_LOG=debug cargo test` produces `[("RUST_LOG", "debug")]`
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<(String, String)>,
    /// Compound structures this command is nested inside, outermost first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nesting: Vec<Nesting>,
}

/// Serde predicate to skip serializing `false` fields.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires &T"
)]
const fn is_false(value: &bool) -> bool {
    !*value
}
