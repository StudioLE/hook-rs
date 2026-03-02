use regex::Regex;
use std::sync::LazyLock;

use crate::prelude::*;

const MAX_CHARS: usize = 1000;
const MAX_LINES: usize = 20;

static PYTHON_INLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bpython3?\s+(-\S+\s+)*(<<|-c\b)").expect("valid regex"));

pub fn check(command: &str) -> Option<CheckResult> {
    if command.is_empty() || !PYTHON_INLINE.is_match(command) {
        return None;
    }
    let char_count = command.len();
    let line_count = command.lines().count();
    if char_count > MAX_CHARS || line_count > MAX_LINES {
        return Some(CheckResult::deny(format!(
            "Inline Python too long ({char_count} chars, {line_count} lines). Write a script to /tmp/ and run it instead."
        )));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn make_heredoc(lines: usize) -> String {
        let mut cmd = "python3 << 'EOF'".to_owned();
        for i in 1..=lines {
            cmd.push('\n');
            cmd.push_str(&format!("print('line {i}')"));
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
        assert_yaml_snapshot!(check(&make_heredoc(25)));
    }

    #[test]
    fn long_heredoc_python2() {
        let cmd = make_heredoc(25).replace("python3", "python");
        assert_yaml_snapshot!(check(&cmd));
    }

    #[test]
    fn long_c_1001_chars() {
        assert_yaml_snapshot!(check(&make_long_c(979)));
    }

    #[test]
    fn short_heredoc_passthrough() {
        assert_eq!(check(&make_heredoc(5)), None);
    }

    #[test]
    fn short_c_passthrough() {
        assert_eq!(check("python3 -c 'print(\"hello\")'"), None);
    }

    #[test]
    fn python_script_passthrough() {
        assert_eq!(check("python3 /tmp/script.py"), None);
    }

    #[test]
    fn python_module_passthrough() {
        assert_eq!(check("python3 -m http.server 8080"), None);
    }

    #[test]
    fn long_non_python_passthrough() {
        let long_bash = format!("bash -c 'echo {}'", "x".repeat(1100));
        assert_eq!(check(&long_bash), None);
    }

    #[test]
    fn ls_passthrough() {
        assert_eq!(check("ls -la"), None);
    }

    #[test]
    fn boundary_exactly_20_lines_passthrough() {
        // make_heredoc(18) -> header + 18 body + EOF = 20 lines
        assert_eq!(check(&make_heredoc(18)), None);
    }

    #[test]
    fn boundary_21_lines_denied() {
        // make_heredoc(19) -> header + 19 body + EOF = 21 lines
        assert_yaml_snapshot!(check(&make_heredoc(19)));
    }

    #[test]
    fn boundary_exactly_1000_chars_passthrough() {
        // make_long_c(978) -> "python3 -c 'print(\"" (22) + 978 + "\")'" (3) = 1000+3? Let's check
        // "python3 -c 'print(\"" = 20 chars, padding, "\")'" = 3 chars  -> 20 + 978 + 3 = 1001? No.
        // Actually: python3 -c 'print("  = 19 chars, then padding, then ")' = 3 chars
        // 19 + 978 + 3 = 1000
        assert_eq!(check(&make_long_c(978)), None);
    }

    #[test]
    fn boundary_1001_chars_denied() {
        assert_yaml_snapshot!(check(&make_long_c(979)));
    }

    #[test]
    fn python_u_flag_long_heredoc() {
        let mut cmd = "python3 -u << 'EOF'".to_owned();
        for i in 1..=25 {
            cmd.push('\n');
            cmd.push_str(&format!("print('line {i}')"));
        }
        cmd.push_str("\nEOF");
        assert_yaml_snapshot!(check(&cmd));
    }
}
