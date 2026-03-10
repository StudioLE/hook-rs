use crate::prelude::*;

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

impl HookOutput {
    pub fn print(&self) {
        let json = serde_json::to_string(&self).expect("should be able to serialize HookOutput");
        println!("{json}");
    }
}

impl From<Outcome> for HookOutput {
    fn from(result: Outcome) -> Self {
        Self {
            hook_specific_output: HookSpecificOutput {
                hook_event_name: "PreToolUse",
                permission_decision: result.decision,
                permission_decision_reason: result.reason,
            },
        }
    }
}
