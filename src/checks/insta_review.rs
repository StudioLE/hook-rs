use crate::prelude::*;

#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    parsed.all_commands().find_map(|cmd| {
        (cmd.name == "cargo"
            && cmd.args.first().is_some_and(|a| a == "insta")
            && cmd.args.get(1).is_some_and(|a| a == "review")
            && cmd.has_heredoc)
            .then(|| {
                CheckResult::deny("Do not fake interactive input to cargo insta review.")
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check(command: &str) -> Option<CheckResult> {
        let parsed = crate::command::parse(command)?;
        super::check(&parsed)
    }

    #[test]
    fn heredoc_single_quoted() {
        assert_yaml_snapshot!(check("cargo insta review 2>&1 <<'EOF'\na\nEOF"));
    }

    #[test]
    fn heredoc_unquoted() {
        assert_yaml_snapshot!(check("cargo insta review <<EOF\na\nEOF"));
    }

    #[test]
    fn heredoc_double_quoted() {
        assert_yaml_snapshot!(check("cargo insta review <<\"EOF\"\na\nEOF"));
    }

    #[test]
    fn heredoc_dash() {
        assert_yaml_snapshot!(check("cargo insta review <<-EOF\na\nEOF"));
    }

    #[test]
    fn plain_review_passthrough() {
        assert_eq!(check("cargo insta review"), None);
    }

    #[test]
    fn insta_test_passthrough() {
        assert_eq!(check("cargo insta test"), None);
    }

    #[test]
    fn insta_test_review_passthrough() {
        assert_eq!(check("cargo insta test --review"), None);
    }
}
