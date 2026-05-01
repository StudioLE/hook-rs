//! Top-level parsed shell command.

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

impl CompleteContext {
    /// Iterate over all [`SimpleContext`] in the parsed command.
    pub fn all_commands(&self) -> impl Iterator<Item = &SimpleContext> {
        self.children.iter().flat_map(|pi| &pi.children)
    }
}
