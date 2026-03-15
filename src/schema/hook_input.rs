//! I/O types for the Claude Code hook protocol.

use crate::prelude::*;
use serde::de::DeserializeOwned;
use std::io;
use std::io::Read;

/// Top-level JSON wrapper from Claude Code, generic over the tool input type.
#[derive(Debug, Deserialize)]
pub struct HookInput<T> {
    /// Tool-specific input fields.
    pub tool_input: T,
}

/// Bash tool input payload.
#[derive(Debug, Deserialize)]
pub struct BashInput {
    /// Shell command to execute.
    pub command: String,
}

/// Read tool input payload.
#[derive(Debug, Deserialize)]
pub struct ReadInput {
    /// File path to read.
    pub file_path: String,
}

impl<T: DeserializeOwned> HookInput<T> {
    /// Read and deserialize hook input JSON from stdin.
    pub fn from_stdin() -> Result<Self, Report<HookError>> {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .change_context(HookError::ReadStdin)?;
        Self::from_json(&input)
    }

    /// Deserialize hook input from a JSON string.
    pub(crate) fn from_json(json: &str) -> Result<Self, Report<HookError>> {
        serde_json::from_str(json).change_context(HookError::DeserializeInput)
    }
}

impl ReadInput {
    /// Create a new [`ReadInput`] for testing.
    #[cfg(test)]
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

/// Errors returned by [`HookInput`] deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum HookError {
    /// Failed to read from stdin.
    #[error("read stdin")]
    ReadStdin,
    /// JSON payload could not be deserialized.
    #[error("deserialize hook input")]
    DeserializeInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_bash_input() {
        let json = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
        let input = HookInput::<BashInput>::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_input.command, "git status");
    }

    #[test]
    fn deserialize_read_input() {
        let json = r#"{"tool_name":"Read","tool_input":{"file_path":"/tmp/foo.rs"}}"#;
        let input = HookInput::<ReadInput>::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_input.file_path, "/tmp/foo.rs");
    }

    #[test]
    fn bash_json_fails_as_read_input() {
        let json = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
        let result = HookInput::<ReadInput>::from_json(json);
        assert!(result.is_err());
    }
}
