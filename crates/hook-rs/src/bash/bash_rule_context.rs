//! Context bundle passed to [`BashRule`] methods and condition functions.

use crate::prelude::*;

/// Context passed to [`BashRule`] for matching and condition evaluation.
///
/// Bundles the simple command, complete command, settings, and working
/// directory into a single parameter.
pub struct BashRuleContext<'a> {
    /// Working directory the command runs in.
    ///
    /// - Taken from the hook input `cwd`
    /// - Absent when Claude Code does not send `cwd`
    pub cwd: Option<String>,
    /// Individual parsed command being evaluated.
    pub simple: &'a SimpleContext,
    /// Complete parsed command including pipelines and chains.
    pub complete: &'a CompleteContext,
    /// User settings.
    pub settings: &'a Settings,
    /// Factory for building path-matching rules.
    pub paths: &'a PathRuleFactory,
}
