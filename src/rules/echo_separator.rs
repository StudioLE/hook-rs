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

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn double_dash_separator() {
        assert_yaml_snapshot!(eval(r#"ls -la && echo "---" && ls -lS"#));
    }

    #[test]
    fn multiple_separators() {
        assert_yaml_snapshot!(eval(r#"cmd1 && echo "---" && cmd2 && echo "---" && cmd3"#));
    }

    #[test]
    fn semicolon_variant() {
        assert_yaml_snapshot!(eval(r#"cmd1 ; echo "---" ; cmd2"#));
    }

    #[test]
    fn single_quoted_separator() {
        assert_yaml_snapshot!(eval("cmd1 && echo '---'"));
    }

    #[test]
    fn equals_separator() {
        assert_yaml_snapshot!(eval(r#"cmd1 && echo "===""#));
    }

    #[test]
    fn labeled_separator() {
        assert_yaml_snapshot!(eval(
            r#"cmd1 && echo "--- Before ---" && cmd2 && echo "--- After ---" && cmd3"#
        ));
    }

    #[test]
    fn long_dash_separator() {
        assert_yaml_snapshot!(eval(r#"cmd1 && echo "------""#));
    }

    #[test]
    fn or_chain_separator() {
        assert_yaml_snapshot!(eval(r#"cmd1 2>&1 || echo "---""#));
    }

    #[test]
    fn real_message_allowed() {
        // make is unmatched but echo matches safe_rules as Allow
        let result = eval(r#"make && echo "Build succeeded""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn status_message_allowed() {
        // cmd is unmatched but echo matches safe_rules as Allow
        let result = eval(r#"cmd && echo "Done processing 5 files""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_piped_passthrough() {
        assert_eq!(eval(r#"true && echo "TEST" | something"#), None);
    }

    #[test]
    fn standalone_echo_passthrough() {
        // echo alone is Allow via safe_rules
        let result = eval(r#"echo "hello""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn standalone_separator_passthrough() {
        // echo alone is Allow via safe_rules
        let result = eval(r#"echo "---""#).expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn ls_passthrough() {
        // ls is Allow via safe_rules
        let result = eval("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_status_passthrough() {
        // git status is Allow via git_approval
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }
}
