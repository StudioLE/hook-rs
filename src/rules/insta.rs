use crate::prelude::*;

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

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn heredoc_single_quoted() {
        assert_yaml_snapshot!(eval("cargo insta review 2>&1 <<'EOF'\na\nEOF"));
    }

    #[test]
    fn heredoc_unquoted() {
        assert_yaml_snapshot!(eval("cargo insta review <<EOF\na\nEOF"));
    }

    #[test]
    fn heredoc_double_quoted() {
        assert_yaml_snapshot!(eval("cargo insta review <<\"EOF\"\na\nEOF"));
    }

    #[test]
    fn heredoc_dash() {
        assert_yaml_snapshot!(eval("cargo insta review <<-EOF\na\nEOF"));
    }

    #[test]
    fn plain_review_passthrough() {
        assert_eq!(eval("cargo insta review"), None);
    }

    #[test]
    fn insta_test_passthrough() {
        assert_eq!(eval("cargo insta test"), None);
    }

    #[test]
    fn insta_test_review_passthrough() {
        assert_eq!(eval("cargo insta test --review"), None);
    }
}
