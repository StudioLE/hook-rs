//! Read tool path settings.

use crate::prelude::*;

/// Glob patterns for auto-allowing Read tool access to trusted paths.
///
/// Patterns starting with `~/` are expanded to `$HOME/`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ReadSettings {
    /// Glob patterns for paths that are safe to read without prompting.
    #[serde(default)]
    pub paths: Vec<String>,
}

impl ReadSettings {
    /// Create [`ReadSettings`] from path patterns.
    #[cfg(test)]
    #[must_use]
    pub fn new(paths: &[&str]) -> Self {
        Self {
            paths: paths.iter().map(|s| String::from(*s)).collect(),
        }
    }
}
