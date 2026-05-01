//! Pipeline of piped commands within a logical chain.

use crate::prelude::*;

/// Multiple [`SimpleCommand`] in a `|` pipeline.
///
/// Example: `git diff --stat HEAD~3 | head -5`
///
/// Commands extracted from `for` loop bodies and command substitutions
/// are flattened into `children` alongside the outer commands. Use
/// [`SimpleContext::nesting`] to distinguish them: top-level commands
/// have an empty `nesting`, while inner commands carry
/// [`Nesting::Substitution`] or [`Nesting::For`]. Inner commands
/// follow the outer command they were extracted from.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PipelineContext {
    /// Logical connector (`&&` or `||`) linking to the previous item.
    ///
    /// `None` for the first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector: Option<Connector>,
    /// Individual commands piped together with `|`.
    ///
    /// Includes both top-level commands and commands extracted from
    /// substitutions or `for` loop bodies. See [`SimpleContext::nesting`].
    pub children: Vec<SimpleContext>,
}
