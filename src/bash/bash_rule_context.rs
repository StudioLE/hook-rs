//! Context bundle passed to [`BashRule`] methods and condition functions.

use crate::prelude::*;

/// Context passed to [`BashRule`] for matching and condition evaluation.
///
/// Bundles the simple command, complete command, and settings into a
/// single parameter.
pub struct BashRuleContext<'a> {
    /// Individual parsed command being evaluated.
    pub simple: &'a SimpleContext,
    /// Complete parsed command including pipelines and chains.
    pub complete: &'a CompleteContext,
    /// User settings.
    pub settings: &'a Settings,
}
