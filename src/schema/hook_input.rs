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

/// Glob tool input payload.
#[derive(Debug, Deserialize)]
pub struct GlobInput {
    /// Glob pattern to match files against.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "deserialized from JSON but only path is used for rule matching"
        )
    )]
    pub pattern: String,
    /// Directory to search in.
    pub path: Option<String>,
}

/// Grep tool input payload.
#[derive(Debug, Deserialize)]
pub struct GrepInput {
    /// Regex pattern to search for.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "deserialized from JSON but only path is used for rule matching"
        )
    )]
    pub pattern: String,
    /// Directory or file path to search in.
    pub path: Option<String>,
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

impl GlobInput {
    /// Create a new [`GlobInput`] for testing.
    #[cfg(test)]
    pub fn new(pattern: impl Into<String>, path: Option<String>) -> Self {
        Self {
            pattern: pattern.into(),
            path,
        }
    }
}

impl GrepInput {
    /// Create a new [`GrepInput`] for testing.
    #[cfg(test)]
    pub fn new(pattern: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            path: Some(path.into()),
        }
    }
}

/// Errors returned by [`HookInput`] deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
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
    fn deserialize_glob_input() {
        let json =
            r#"{"tool_name":"Glob","tool_input":{"pattern":"**/*.rs","path":"/opt/readonly/src"}}"#;
        let input = HookInput::<GlobInput>::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_input.pattern, "**/*.rs");
        assert_eq!(input.tool_input.path.as_deref(), Some("/opt/readonly/src"));
    }

    #[test]
    fn deserialize_glob_input_without_path() {
        let json = r#"{"tool_name":"Glob","tool_input":{"pattern":"**/*.rs"}}"#;
        let input = HookInput::<GlobInput>::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_input.pattern, "**/*.rs");
        assert!(input.tool_input.path.is_none());
    }

    #[test]
    fn deserialize_grep_input() {
        let json =
            r#"{"tool_name":"Grep","tool_input":{"pattern":"needle","path":"/tmp/project"}}"#;
        let input = HookInput::<GrepInput>::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_input.pattern, "needle");
        assert_eq!(input.tool_input.path.as_deref(), Some("/tmp/project"));
    }

    #[test]
    fn deserialize_grep_input_without_path() {
        let json = r#"{"tool_name":"Grep","tool_input":{"pattern":"needle"}}"#;
        let input = HookInput::<GrepInput>::from_json(json).expect("should deserialize");
        assert_eq!(input.tool_input.pattern, "needle");
        assert!(input.tool_input.path.is_none());
    }

    #[test]
    fn bash_json_fails_as_read_input() {
        let json = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
        let result = HookInput::<ReadInput>::from_json(json);
        assert!(result.is_err());
    }
}
