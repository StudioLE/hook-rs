//! Rule for `awk` commands.

use crate::prelude::*;

/// Rule for `awk` commands.
pub fn awk() -> BashRule {
    BashRule::new(
        "awk",
        "awk",
        Outcome::deny(
            "awk is blocked. Alternatives: built in `Read` tool, `cut`, `grep`, `head`, `tail`, or `wc -l`",
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn awk_field_extract() {
        let outcome = evaluate_expect_outcome("awk '{print $2}' file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_line_range() {
        let outcome = evaluate_expect_outcome("awk 'NR>=10 && NR<=20' file.rs");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_in_pipe() {
        let outcome = evaluate_expect_outcome("ps aux | awk '{print $1}'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_chained() {
        let outcome = evaluate_expect_outcome("awk '{print}' a.txt; awk '{print}' b.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_system() {
        let outcome = evaluate_expect_outcome("awk 'BEGIN { system(\"rm -rf /\") }'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_dash_f() {
        let outcome = evaluate_expect_outcome("awk -f script.awk file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn awk_bare() {
        let outcome = evaluate_expect_outcome("awk");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn echo_awk() {
        let outcome = evaluate_expect_outcome("echo awk is denied");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_awk() {
        let outcome = evaluate_expect_outcome("rg awk .");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
