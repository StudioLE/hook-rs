use regex::Regex;
use std::sync::LazyLock;

use crate::prelude::*;

static INSTA_HEREDOC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"cargo\s+insta\s+review.*<<").expect("valid regex"));

pub fn check(command: &str) -> Option<CheckResult> {
    INSTA_HEREDOC
        .is_match(command)
        .then(|| CheckResult::deny("Do not fake interactive input to cargo insta review."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn heredoc_single_quoted() {
        assert_yaml_snapshot!(check("cargo insta review 2>&1 <<'EOF'\n   a\n   EOF"));
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
