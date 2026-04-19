//! Rules for Python commands.

use crate::prelude::*;

const REASON: &str = "Python is not permitted. Use shell tools, jq, yq, or Rust instead.";

/// Rules for all Python commands.
pub fn python_rules() -> Vec<BashRule> {
    vec![python(), python3()]
}

/// Rule for `python` commands.
fn python() -> BashRule {
    BashRule::new("python", "python", Outcome::deny(REASON))
}

/// Rule for `python3` commands.
fn python3() -> BashRule {
    BashRule::new("python3", "python3", Outcome::deny(REASON))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn python_inline() {
        let outcome = evaluate_expect_outcome("python -c 'print(1)'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_inline() {
        let outcome = evaluate_expect_outcome("python3 -c 'print(1)'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_script() {
        let outcome = evaluate_expect_outcome("python3 /tmp/script.py");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_module() {
        let outcome = evaluate_expect_outcome("python3 -m http.server 8080");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_heredoc() {
        let outcome = evaluate_expect_outcome("python3 << 'EOF'\nprint('hello')\nEOF");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_bare() {
        let outcome = evaluate_expect_outcome("python3");
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
