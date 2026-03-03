//! I/O types for the Claude Code hook protocol.

use serde::{Deserialize, Serialize};

use crate::check::{CheckResult, Decision};

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

/// Top-level JSON output returned to Claude Code.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    /// Hook-specific output containing the decision.
    pub hook_specific_output: HookSpecificOutput,
}

/// Detailed hook output with event metadata.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput {
    /// Event name identifying the hook type.
    pub hook_event_name: &'static str,
    /// The permission decision.
    pub permission_decision: Decision,
    /// Human-readable reason for the decision.
    pub permission_decision_reason: String,
}

impl From<CheckResult> for HookOutput {
    fn from(result: CheckResult) -> Self {
        Self {
            hook_specific_output: HookSpecificOutput {
                hook_event_name: "PreToolUse",
                permission_decision: result.decision,
                permission_decision_reason: result.reason,
            },
        }
    }
}
