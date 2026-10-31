//! Git path classification settings.

use crate::prelude::*;

/// Git path classification for `git -C` operations.
///
/// Ordered glob patterns following `.gitignore` semantics:
///
/// - Evaluated top-to-bottom, last match wins
/// - Prefix with `!` to negate (untrust)
/// - Paths matching no pattern are untrusted
/// - Supports tilde expansion (`~/repos/**`)
///
/// ```yaml
/// git:
///   paths:
///     - /home/user/repos/**
///     - !/home/user/repos/forked/**
///     - /home/user/repos/forked/this
/// ```
///
/// See CVE-2025-59536 and CVE-2026-21852.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GitSettings {
    /// Glob patterns for `git -C` trust classification.
    ///
    /// - Last matching pattern wins
    /// - Prefix with `!` to negate
    /// - Supports tilde expansion (`~/repos/**`)
    #[serde(default)]
    pub paths: Vec<String>,
}

impl GitSettings {
    /// Create [`GitSettings`] from path patterns.
    #[cfg(test)]
    #[must_use]
    pub fn new(paths: &[&str]) -> Self {
        Self {
            paths: paths.iter().map(|s| String::from(*s)).collect(),
        }
    }
}
