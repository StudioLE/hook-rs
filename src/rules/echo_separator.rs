//! Deny rule for chained echo separators like `cmd && echo "---" && cmd`.

use crate::prelude::*;

/// Deny compound commands that use echo with `---` or `===` as visual separators.
pub fn echo_separator_rules() -> Vec<CompleteRule> {
    vec![echo_separator()]
}

/// Deny chained echo separators.
fn echo_separator() -> CompleteRule {
    CompleteRule {
        id: "echo_separator".to_owned(),
        condition: Some(has_chained_echo_separator),
        outcome: Outcome::deny("Chained echo separators are blocked. Run each command separately."),
    }
}

fn has_chained_echo_separator(parsed: &CompleteContext) -> bool {
    parsed.children.iter().any(|pi| {
        pi.connector.is_some()
            && pi.children.first().is_some_and(|cmd| {
                cmd.name == "echo"
                    && cmd.args.first().is_some_and(|arg| {
                        let unquoted = unquote_str(arg);
                        unquoted.starts_with("---") || unquoted.starts_with("===")
                    })
            })
    })
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn double_dash_separator() {
        let outcome = evaluate_expect_outcome(r#"ls -la && echo "---" && ls -lS"#);
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn multiple_separators() {
        let outcome =
            evaluate_expect_outcome(r#"cmd1 && echo "---" && cmd2 && echo "---" && cmd3"#);
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn semicolon_variant() {
        let outcome = evaluate_expect_outcome(r#"cmd1 ; echo "---" ; cmd2"#);
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn single_quoted_separator() {
        let outcome = evaluate_expect_outcome("cmd1 && echo '---'");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn equals_separator() {
        let outcome = evaluate_expect_outcome(r#"cmd1 && echo "===""#);
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn labeled_separator() {
        let outcome = evaluate_expect_outcome(
            r#"cmd1 && echo "--- Before ---" && cmd2 && echo "--- After ---" && cmd3"#,
        );
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn long_dash_separator() {
        let outcome = evaluate_expect_outcome(r#"cmd1 && echo "------""#);
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn or_chain_separator() {
        let outcome = evaluate_expect_outcome(r#"cmd1 2>&1 || echo "---""#);
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn real_message_allowed() {
        // make is unmatched but echo matches safe_rules as Allow
        let reason = evaluate_expect_skip(r#"make && echo "Build succeeded""#);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn status_message_allowed() {
        // cmd is unmatched but echo matches safe_rules as Allow
        let reason = evaluate_expect_skip(r#"cmd && echo "Done processing 5 files""#);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn echo_piped_passthrough() {
        let reason = evaluate_expect_skip(r#"true && echo "TEST" | something"#);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn standalone_echo_passthrough() {
        // echo alone is Allow via safe_rules
        let outcome = evaluate_expect_outcome(r#"echo "hello""#);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn standalone_separator_passthrough() {
        // echo alone is Allow via safe_rules
        let outcome = evaluate_expect_outcome(r#"echo "---""#);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn ls_passthrough() {
        // ls is Allow via safe_rules
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_status_passthrough() {
        // git status is Allow via git_approval
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
