//! Deny commands separated by `;`.

use crate::prelude::*;

/// Deny sequential commands separated by `;`.
pub fn semicolon_rule(complete: &CompleteContext) -> Option<Outcome> {
    contains_semicolon(complete).then(|| Outcome::deny(
            "Sequential commands with `;` are blocked. Use `&&` for related commands or run each separately",
        ))
}

/// Check if any pipeline in the command uses `;` as a separator.
fn contains_semicolon(complete: &CompleteContext) -> bool {
    complete
        .children
        .iter()
        .any(|p| matches!(p.connector, Some(Connector::Semi)))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    /// Two commands separated by `;`
    #[test]
    fn semicolon_two_commands() {
        let outcome = evaluate_expect_outcome("git status ; echo hi");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    /// Three commands separated by `;`
    #[test]
    fn semicolon_three_commands() {
        let outcome = evaluate_expect_outcome("git status ; echo hi ; ls");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    /// For loop semicolons are syntactic, not separators
    #[test]
    fn semicolon_for_loop() {
        let outcome = evaluate_expect_outcome("for f in *.txt; do echo $f; done");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// `&&` followed by a for loop has no `Connector::Semi`
    #[test]
    fn semicolon_and_then_for_loop() {
        let outcome = evaluate_expect_outcome("git status && for f in *.txt; do echo $f; done");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// `&&` is not `;`
    #[test]
    fn semicolon_and_connector() {
        let outcome = evaluate_expect_outcome("git status && echo hi");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
