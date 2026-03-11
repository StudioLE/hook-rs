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
    /// Whether the command contains a `for` loop.
    pub has_for: bool,
}

/// Multiple [`SimpleCommand`] in a `|` pipeline.
///
/// Example: `git diff --stat HEAD~3 | head -5`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PipelineContext {
    /// Logical connector (`&&` or `||`) linking to the previous item.
    ///
    /// `None` for the first.
    pub connector: Option<Connector>,
    /// Individual commands piped together with `|`.
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
    pub args: Vec<String>,
    /// Whether the command has a heredoc redirect.
    pub has_heredoc: bool,
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

    /// True if the command is a single simple command with no chaining or piping.
    #[must_use]
    pub fn is_standalone(&self) -> bool {
        self.children.len() == 1
            && self
                .children
                .first()
                .is_some_and(|pi| pi.children.len() == 1)
    }
}
