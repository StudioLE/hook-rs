use crate::prelude::*;

/// Result of evaluating a single check against a parsed command.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Outcome {
    /// The permission decision.
    pub decision: Decision,
    /// Human-readable explanation for the decision.
    pub reason: String,
}

/// Outcome of a hook check.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    #[default]
    Ask,
    Deny,
}

impl Outcome {
    /// Create an allow result with the given reason.
    pub fn allow(reason: impl Into<String>) -> Self {
        Self {
            decision: Decision::Allow,
            reason: reason.into(),
        }
    }

    /// Create a deny result with the given reason.
    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            decision: Decision::Deny,
            reason: reason.into(),
        }
    }

    /// Create an ask result with the given reason.
    pub fn ask(reason: impl Into<String>) -> Self {
        Self {
            decision: Decision::Ask,
            reason: reason.into(),
        }
    }

    pub fn print_hook_output(self) {
        HookOutput::from(self).print();
    }
}

impl Display for Decision {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Deny => write!(f, "deny"),
            Self::Ask => write!(f, "ask"),
        }
    }
}
