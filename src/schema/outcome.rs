//! Outcome and decision types for rule evaluation.

use crate::prelude::*;

/// Result of evaluating a single rule against a parsed command.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Outcome {
    /// The permission decision.
    pub decision: Decision,
    /// Human-readable explanation for the decision.
    pub reason: String,
}

/// Permission decision for a hook rule.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Error, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    #[error("Allow")]
    Allow,
    #[default]
    #[error("Ask")]
    Ask,
    #[error("Deny")]
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

    /// Create an ask result from an error.
    pub fn error<T: Error>(error: Report<T>) -> Self {
        Self {
            decision: Decision::Ask,
            reason: format!("ERROR: {error:?}"),
        }
    }

    /// Create an outcome by joining multiple reason strings with newlines.
    pub(crate) fn combined(decision: Decision, reasons: &[String]) -> Self {
        if reasons.is_empty() {
            unreachable!("it should not be possible to create an outcome with no reasons");
        }
        Self {
            decision,
            reason: reasons.join("\n"),
        }
    }

    /// Serialize this outcome as [`HookOutput`] JSON and print to stdout.
    pub fn print_hook_output(self) {
        HookOutput::from(self).print();
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.decision, self.reason)
    }
}
