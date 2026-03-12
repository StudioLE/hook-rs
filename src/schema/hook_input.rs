//! I/O types for the Claude Code hook protocol.

use crate::prelude::*;
use std::io;
use std::io::Read;

/// Top-level JSON input from Claude Code.
#[derive(Debug, Deserialize)]
pub struct HookInput {
    /// Name of the tool being invoked (e.g. "Bash", "Read").
    pub tool_name: String,
    /// Tool-specific input fields.
    pub tool_input: ToolInput,
}

/// Tool input payload.
///
/// Fields are optional to support multiple tool shapes.
#[derive(Debug, Deserialize)]
pub struct ToolInput {
    /// Shell command, present for Bash tool calls.
    pub command: Option<String>,
    /// File path, present for Read tool calls.
    pub file_path: Option<String>,
}

impl HookInput {
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
        let input = HookInput::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_name, "Bash");
        assert_eq!(input.tool_input.command.as_deref(), Some("git status"));
        assert!(input.tool_input.file_path.is_none());
    }

    #[test]
    fn deserialize_read_input() {
        let json = r#"{"tool_name":"Read","tool_input":{"file_path":"/tmp/foo.rs"}}"#;
        let input = HookInput::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_name, "Read");
        assert_eq!(input.tool_input.file_path.as_deref(), Some("/tmp/foo.rs"));
        assert!(input.tool_input.command.is_none());
    }

    #[test]
    fn deserialize_unknown_tool() {
        let json =
            r#"{"tool_name":"Write","tool_input":{"file_path":"/tmp/foo.rs","content":"hi"}}"#;
        let input = HookInput::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_name, "Write");
    }
}
