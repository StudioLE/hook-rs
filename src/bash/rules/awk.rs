//! Rule for `awk` commands.

use crate::prelude::*;

/// Rule for `awk` commands.
pub fn awk() -> BashRule {
    BashRule::new(
        "awk",
        "awk",
        Outcome::deny(
            "`awk` is blocked. Alternatives: built-in `Read` tool, `cut`, `grep`, `head`, `tail`, `wc -l`",
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn awk_field_extract() {
        let result = eval_rules(vec![awk()], "awk '{print $2}' file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_line_range() {
        let result = eval_rules(vec![awk()], "awk 'NR>=10 && NR<=20' file.rs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_in_pipe() {
        let result = eval_rules(vec![awk()], "ps aux | awk '{print $1}'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_chained() {
        let result = eval_rules(vec![awk()], "awk '{print}' a.txt; awk '{print}' b.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_system() {
        let result = eval_rules(vec![awk()], "awk 'BEGIN { system(\"rm -rf /\") }'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_dash_f() {
        let result = eval_rules(vec![awk()], "awk -f script.awk file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_bare() {
        let result = eval_rules(vec![awk()], "awk");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn echo_awk() {
        let result = eval_rules(vec![awk()], "echo awk is denied");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_awk() {
        let result = eval_rules(vec![awk()], "rg awk .");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
