//! Worktree path classification settings.

use crate::prelude::*;

/// Worktree path classification for `git worktree add` operations.
///
/// Ordered glob patterns following `.gitignore` semantics:
///
/// - Evaluated top-to-bottom, last match wins
/// - Prefix with `!` to negate (untrust)
/// - Paths matching no pattern are untrusted
/// - Supports tilde expansion (`~/worktrees/**`)
///
/// ```yaml
/// worktrees:
///   paths:
///     - /home/user/worktrees/**
/// ```
#[derive(Clone, Debug, Default, Deserialize)]
pub struct WorktreeSettings {
    /// Glob patterns for trusted worktree target directories.
    ///
    /// - Last matching pattern wins
    /// - Prefix with `!` to negate
    /// - Supports tilde expansion (`~/worktrees/**`)
    #[serde(default)]
    pub paths: Vec<String>,
}

impl WorktreeSettings {
    /// Create [`WorktreeSettings`] from path patterns.
    #[cfg(test)]
    #[must_use]
    pub fn new(paths: &[&str]) -> Self {
        Self {
            paths: paths.iter().map(|s| String::from(*s)).collect(),
        }
    }
}
