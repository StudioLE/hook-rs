//! Deny rule for faking interactive input to `cargo insta review`.

use crate::prelude::*;

/// Deny `cargo insta review` when used with heredoc input.
pub fn insta_rules() -> Vec<SimpleRule> {
    vec![SimpleRule {
        prefix: "cargo insta".to_owned(),
        condition: Some(is_review_with_heredoc),
        outcome: Outcome::deny("Do not fake interactive input to cargo insta review."),
        ..Default::default()
    }]
}

fn is_review_with_heredoc(cmd: &SimpleContext) -> bool {
    cmd.args.get(1).is_some_and(|a| a == "review") && cmd.has_heredoc
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn heredoc_single_quoted() {
        let result = evaluate("cargo insta review 2>&1 <<'EOF'\na\nEOF");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn heredoc_unquoted() {
        let result = evaluate("cargo insta review <<EOF\na\nEOF");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn heredoc_double_quoted() {
        let result = evaluate("cargo insta review <<\"EOF\"\na\nEOF");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn heredoc_dash() {
        let result = evaluate("cargo insta review <<-EOF\na\nEOF");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn plain_review_passthrough() {
        assert_eq!(evaluate("cargo insta review"), None);
    }

    #[test]
    fn insta_test_passthrough() {
        assert_eq!(evaluate("cargo insta test"), None);
    }

    #[test]
    fn insta_test_review_passthrough() {
        assert_eq!(evaluate("cargo insta test --review"), None);
    }
}
