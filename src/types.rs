use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub tool_input: ToolInput,
}

#[derive(Debug, Deserialize)]
pub struct ToolInput {
    pub command: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Deny,
    Ask,
}

impl fmt::Display for Decision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Deny => write!(f, "deny"),
            Self::Ask => write!(f, "ask"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CheckResult {
    pub decision: Decision,
    pub reason: String,
}

impl CheckResult {
    pub fn allow(reason: impl Into<String>) -> Self {
        Self {
            decision: Decision::Allow,
            reason: reason.into(),
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            decision: Decision::Deny,
            reason: reason.into(),
        }
    }

    pub fn ask(reason: impl Into<String>) -> Self {
        Self {
            decision: Decision::Ask,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput {
    pub hook_event_name: &'static str,
    pub permission_decision: Decision,
    pub permission_decision_reason: String,
}

#[derive(Debug)]
pub struct ParsedCommand {
    pub raw: String,
    pub and_or_lists: Vec<AndOrContext>,
}

#[derive(Debug)]
pub struct AndOrContext {
    pub items: Vec<PipelineItem>,
}

#[derive(Debug)]
pub struct PipelineItem {
    pub connector: Option<Connector>,
    pub commands: Vec<CommandContext>,
}

#[derive(Debug)]
pub struct CommandContext {
    pub name: String,
    pub args: Vec<String>,
    pub has_heredoc: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connector {
    And,
    Or,
}

impl ParsedCommand {
    pub fn all_commands(&self) -> impl Iterator<Item = &CommandContext> {
        self.and_or_lists
            .iter()
            .flat_map(|aol| &aol.items)
            .flat_map(|pi| &pi.commands)
    }

    #[must_use]
    pub fn is_standalone(&self) -> bool {
        self.and_or_lists
            .first()
            .is_some_and(|aol| {
                aol.items.len() == 1
                    && aol.items.first().is_some_and(|pi| pi.commands.len() == 1)
            })
            && self.and_or_lists.len() == 1
    }
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
