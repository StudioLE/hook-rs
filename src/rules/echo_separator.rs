use crate::prelude::*;

pub fn echo_separator_rules() -> Vec<CompleteRule> {
    vec![CompleteRule {
        condition: Some(has_chained_echo_separator),
        outcome: Outcome::deny("Chained echo separators are blocked. Run each command separately."),
    }]
}

fn has_chained_echo_separator(parsed: &CompleteContext) -> bool {
    parsed.children.iter().any(|pi| {
        pi.connector.is_some()
            && pi.children.first().is_some_and(|cmd| {
                cmd.name == "echo"
                    && cmd.args.first().is_some_and(|arg| {
                        let unquoted = unquote(arg);
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
        let result = evaluate(r#"ls -la && echo "---" && ls -lS"#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn multiple_separators() {
        let result = evaluate(r#"cmd1 && echo "---" && cmd2 && echo "---" && cmd3"#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn semicolon_variant() {
        let result = evaluate(r#"cmd1 ; echo "---" ; cmd2"#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn single_quoted_separator() {
        let result = evaluate("cmd1 && echo '---'");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn equals_separator() {
        let result = evaluate(r#"cmd1 && echo "===""#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn labeled_separator() {
        let result =
            evaluate(r#"cmd1 && echo "--- Before ---" && cmd2 && echo "--- After ---" && cmd3"#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn long_dash_separator() {
        let result = evaluate(r#"cmd1 && echo "------""#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn or_chain_separator() {
        let result = evaluate(r#"cmd1 2>&1 || echo "---""#);
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn real_message_allowed() {
        // make is unmatched but echo matches safe_rules as Allow
        let result = evaluate(r#"make && echo "Build succeeded""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn status_message_allowed() {
        // cmd is unmatched but echo matches safe_rules as Allow
        let result = evaluate(r#"cmd && echo "Done processing 5 files""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_piped_passthrough() {
        assert_eq!(evaluate(r#"true && echo "TEST" | something"#), None);
    }

    #[test]
    fn standalone_echo_passthrough() {
        // echo alone is Allow via safe_rules
        let result = evaluate(r#"echo "hello""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn standalone_separator_passthrough() {
        // echo alone is Allow via safe_rules
        let result = evaluate(r#"echo "---""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn ls_passthrough() {
        // ls is Allow via safe_rules
        let result = evaluate("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_status_passthrough() {
        // git status is Allow via git_approval
        let result = evaluate("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }
}
