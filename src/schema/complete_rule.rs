use crate::prelude::*;

pub struct CompleteRule {
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
