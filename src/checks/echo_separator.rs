//! Check for chained echo separator patterns.

use crate::prelude::*;

/// Deny chained echo separator patterns like `echo "---"`.
#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    // After && or ||
    for aol in &parsed.and_or_lists {
        for pi in &aol.items {
            if pi.connector.is_some() && has_echo_separator(pi) {
                return Some(CheckResult::deny(
                    "Chained echo separators are blocked. Run each command separately.",
                ));
            }
        }
    }
    // After ; (non-first and_or_list)
    for aol in parsed.and_or_lists.iter().skip(1) {
        for pi in &aol.items {
            if has_echo_separator(pi) {
                return Some(CheckResult::deny(
                    "Chained echo separators are blocked. Run each command separately.",
                ));
            }
        }
    }
    None
}

fn has_echo_separator(pi: &PipelineItem) -> bool {
    pi.commands.first().is_some_and(|cmd| {
        cmd.name == "echo"
            && cmd.args.first().is_some_and(|arg| {
                let unquoted = unquote(arg);
                unquoted.starts_with("---") || unquoted.starts_with("===")
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check(command: &str) -> Option<CheckResult> {
        let parsed = parse(command)?;
        super::check(&parsed)
    }

    #[test]
    fn double_dash_separator() {
        assert_yaml_snapshot!(check(r#"ls -la && echo "---" && ls -lS"#));
    }

    #[test]
    fn multiple_separators() {
        assert_yaml_snapshot!(check(r#"cmd1 && echo "---" && cmd2 && echo "---" && cmd3"#));
    }

    #[test]
    fn semicolon_variant() {
        assert_yaml_snapshot!(check(r#"cmd1 ; echo "---" ; cmd2"#));
    }

    #[test]
    fn single_quoted_separator() {
        assert_yaml_snapshot!(check("cmd1 && echo '---'"));
    }

    #[test]
    fn equals_separator() {
        assert_yaml_snapshot!(check(r#"cmd1 && echo "===""#));
    }

    #[test]
    fn labeled_separator() {
        assert_yaml_snapshot!(check(
            r#"cmd1 && echo "--- Before ---" && cmd2 && echo "--- After ---" && cmd3"#
        ));
    }

    #[test]
    fn long_dash_separator() {
        assert_yaml_snapshot!(check(r#"cmd1 && echo "------""#));
    }

    #[test]
    fn or_chain_separator() {
        assert_yaml_snapshot!(check(r#"cmd1 2>&1 || echo "---""#));
    }

    #[test]
    fn real_message_passthrough() {
        assert_eq!(check(r#"make && echo "Build succeeded""#), None);
    }

    #[test]
    fn status_message_passthrough() {
        assert_eq!(check(r#"cmd && echo "Done processing 5 files""#), None);
    }

    #[test]
    fn echo_piped_passthrough() {
        assert_eq!(check(r#"true && echo "TEST" | something"#), None);
    }

    #[test]
    fn standalone_echo_passthrough() {
        assert_eq!(check(r#"echo "hello""#), None);
    }

    #[test]
    fn standalone_separator_passthrough() {
        assert_eq!(check(r#"echo "---""#), None);
    }

    #[test]
    fn ls_passthrough() {
        assert_eq!(check("ls -la"), None);
    }

    #[test]
    fn git_status_passthrough() {
        assert_eq!(check("git status"), None);
    }
}
