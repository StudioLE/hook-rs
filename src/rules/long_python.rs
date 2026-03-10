use crate::prelude::*;

const MAX_CHARS: usize = 1000;
const MAX_LINES: usize = 20;

pub fn long_python_rules() -> Vec<CompleteRule> {
    vec![CompleteRule {
        condition: Some(is_long_inline_python),
        outcome: Outcome::deny(format!(
            "Inline Python too long (must be < {MAX_CHARS} chars and < {MAX_LINES} lines). Write a script to /tmp/ and run it instead."
        )),
    }]
}

fn is_long_inline_python(parsed: &CompleteContext) -> bool {
    let has_inline_python = parsed.all_commands().any(|cmd| {
        (cmd.name == "python" || cmd.name == "python3")
            && (cmd.args.iter().any(|a| a == "-c") || cmd.has_heredoc)
    });
    if !has_inline_python {
        return false;
    }
    parsed.raw.len() > MAX_CHARS || parsed.raw.lines().count() > MAX_LINES
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    fn make_heredoc(lines: usize) -> String {
        use std::fmt::Write;
        let mut cmd = "python3 << 'EOF'".to_owned();
        for i in 1..=lines {
            write!(cmd, "\nprint('line {i}')").expect("write to String should not fail");
        }
        cmd.push_str("\nEOF");
        cmd
    }

    fn make_long_c(padding_len: usize) -> String {
        let padding: String = "x".repeat(padding_len);
        format!("python3 -c 'print(\"{padding}\")'")
    }

    #[test]
    fn long_heredoc_25_lines() {
        assert_yaml_snapshot!(eval(&make_heredoc(25)));
    }

    #[test]
    fn long_heredoc_python2() {
        let cmd = make_heredoc(25).replace("python3", "python");
        assert_yaml_snapshot!(eval(&cmd));
    }

    #[test]
    fn long_c_1001_chars() {
        assert_yaml_snapshot!(eval(&make_long_c(979)));
    }

    #[test]
    fn short_heredoc_passthrough() {
        assert_eq!(eval(&make_heredoc(5)), None);
    }

    #[test]
    fn short_c_passthrough() {
        assert_eq!(eval("python3 -c 'print(\"hello\")'"), None);
    }

    #[test]
    fn python_script_passthrough() {
        assert_eq!(eval("python3 /tmp/script.py"), None);
    }

    #[test]
    fn python_module_passthrough() {
        assert_eq!(eval("python3 -m http.server 8080"), None);
    }

    #[test]
    fn long_non_python_passthrough() {
        let long_bash = format!("bash -c 'echo {}'", "x".repeat(1100));
        assert_eq!(eval(&long_bash), None);
    }

    #[test]
    fn ls_passthrough() {
        // ls is Allow via safe_rules
        let result = eval("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn boundary_exactly_20_lines_passthrough() {
        assert_eq!(eval(&make_heredoc(18)), None);
    }

    #[test]
    fn boundary_21_lines_denied() {
        assert_yaml_snapshot!(eval(&make_heredoc(19)));
    }

    #[test]
    fn boundary_exactly_1000_chars_passthrough() {
        assert_eq!(eval(&make_long_c(978)), None);
    }

    #[test]
    fn boundary_1001_chars_denied() {
        assert_yaml_snapshot!(eval(&make_long_c(979)));
    }

    #[test]
    fn python_u_flag_long_heredoc() {
        use std::fmt::Write;
        let mut cmd = "python3 -u << 'EOF'".to_owned();
        for i in 1..=25 {
            write!(cmd, "\nprint('line {i}')").expect("write to String should not fail");
        }
        cmd.push_str("\nEOF");
        assert_yaml_snapshot!(eval(&cmd));
    }
}
