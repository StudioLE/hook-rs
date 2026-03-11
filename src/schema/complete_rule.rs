//! Condition-based rule matching complete parsed commands.

use crate::prelude::*;

/// Rule that matches a [`CompleteContext`] by a condition function.
pub struct CompleteRule {
    /// Unique identifier for this rule.
    #[expect(dead_code, reason = "id for planned matching")]
    pub id: String,
    /// Only match if the command satisfies this condition.
    pub condition: Option<fn(&CompleteContext) -> bool>,
    /// Outcome if the command matches.
    pub outcome: Outcome,
}

impl CompleteRule {
    /// Check if this rule matches the given command.
    #[must_use]
    pub fn matches(&self, parsed: &CompleteContext) -> bool {
        self.condition.as_ref().is_some_and(|f| f(parsed))
    }
}
