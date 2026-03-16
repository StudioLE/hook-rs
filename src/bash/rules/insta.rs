//! Deny rule for faking interactive input to `cargo insta review`.

use crate::prelude::*;

/// Deny `cargo insta review` when used with heredoc input.
pub fn insta_rules() -> Vec<BashRule> {
    vec![cargo_insta_review__heredoc()]
}

/// Deny `cargo insta review` with heredoc input.
fn cargo_insta_review__heredoc() -> BashRule {
    BashRule {
        id: "cargo_insta_review__heredoc".to_owned(),
        command: "cargo insta review".to_owned(),
        condition: Some(|simple, _, _| simple.has_heredoc),
        outcome: Outcome::deny("Do not fake interactive input to cargo insta review."),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn heredoc_single_quoted() {
        let outcome = evaluate_expect_outcome("cargo insta review 2>&1 <<'EOF'\na\nEOF");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn heredoc_unquoted() {
        let outcome = evaluate_expect_outcome("cargo insta review <<EOF\na\nEOF");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn heredoc_double_quoted() {
        let outcome = evaluate_expect_outcome("cargo insta review <<\"EOF\"\na\nEOF");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn heredoc_dash() {
        let outcome = evaluate_expect_outcome("cargo insta review <<-EOF\na\nEOF");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn plain_review_passthrough() {
        let reason = evaluate_expect_skip("cargo insta review");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn insta_test_passthrough() {
        let reason = evaluate_expect_skip("cargo insta test");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn insta_test_review_passthrough() {
        let reason = evaluate_expect_skip("cargo insta test --review");
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
