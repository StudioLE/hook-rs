//! I/O types for the Claude Code hook protocol.

use crate::prelude::*;
use std::io;
use std::io::Read;

/// Top-level JSON input from Claude Code.
#[derive(Debug, Deserialize)]
pub struct HookInput {
    /// Tool-specific input containing the command string.
    pub tool_input: ToolInput,
}

/// Tool input payload.
#[derive(Debug, Deserialize)]
pub struct ToolInput {
    /// Shell command to evaluate.
    pub command: String,
}

impl HookInput {
    /// Read and deserialize hook input JSON from stdin.
    pub fn from_stdin() -> Result<Self, Report<HookError>> {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .change_context(HookError::ReadStdin)?;
        Self::from_json(input)
    }

    fn from_json(json: String) -> Result<Self, Report<HookError>> {
        serde_json::from_str(&json).change_context(HookError::DeserializeInput)
    }
}

/// Errors returned by [`HookInput`] deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum HookError {
    #[error("read stdin")]
    ReadStdin,
    #[error("deserialize hook input")]
    DeserializeInput,
}
