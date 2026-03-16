//! Shell command parser and AST types.
//!
//! - Tokenizes and parses shell commands via [`brush_parser`]
//! - Produces a [`CompleteContext`] AST for check evaluation

use crate::prelude::*;

/// Complete command context.
///
/// Example: `cd path/to/repo && git diff --stat HEAD~3 | head -5`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompleteContext {
    /// Original command string.
    pub raw: String,
    /// Command pipelines split by `&&`, `||`, or `;`.
    pub children: Vec<PipelineContext>,
}

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
    /// Compound structures this command is nested inside, outermost first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nesting: Vec<Nesting>,
}

/// Compound structure that a command can be nested inside.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Nesting {
    /// Inside a `for` loop body.
    For,
    /// Inside a command substitution.
    Substitution,
}

/// Logical connector between pipeline items.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Connector {
    And,
    Or,
    Semi,
}

impl CompleteContext {
    /// Iterate over all [`SimpleContext`] in the parsed command.
    pub fn all_commands(&self) -> impl Iterator<Item = &SimpleContext> {
        self.children.iter().flat_map(|pi| &pi.children)
    }
}

/// Serde predicate to skip serializing `false` fields.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires &T"
)]
const fn is_false(value: &bool) -> bool {
    !*value
}
