//! Deny rule for excessively long inline Python commands.

use crate::prelude::*;

const MAX_CHARS: usize = 1000;
const MAX_LINES: usize = 20;

/// Deny inline Python exceeding length or line count thresholds.
pub fn long_python_rules() -> Vec<BashRule> {
    vec![python__long_inline(), python3__long_inline()]
}

/// Deny excessively long inline `python` commands.
fn python__long_inline() -> BashRule {
    BashRule {
        condition: Some(is_long_inline),
        ..BashRule::new(
            "python__long_inline",
            "python",
            Outcome::deny(format!(
                "Inline Python too long (must be < {MAX_CHARS} chars and < {MAX_LINES} lines). Write a script to /tmp/ and run it instead."
            )),
        )
    }
}

/// Deny excessively long inline `python3` commands.
fn python3__long_inline() -> BashRule {
    BashRule {
        condition: Some(is_long_inline),
        ..BashRule::new(
            "python3__long_inline",
            "python3",
            Outcome::deny(format!(
                "Inline Python too long (must be < {MAX_CHARS} chars and < {MAX_LINES} lines). Write a script to /tmp/ and run it instead."
            )),
        )
    }
}

fn is_long_inline(
    simple: &SimpleContext,
    complete: &CompleteContext,
    _settings: &Settings,
) -> bool {
    let has_inline = simple.args.iter().any(|a| a == "-c") || simple.has_heredoc;
    has_inline && (complete.raw.len() > MAX_CHARS || complete.raw.lines().count() > MAX_LINES)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

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
        let outcome = evaluate_expect_outcome(&make_heredoc(25));
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn long_heredoc_python2() {
        let cmd = make_heredoc(25).replace("python3", "python");
        let outcome = evaluate_expect_outcome(&cmd);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn long_c_1001_chars() {
        let outcome = evaluate_expect_outcome(&make_long_c(979));
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn short_heredoc_passthrough() {
        let reason = evaluate_expect_skip(&make_heredoc(5));
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn short_c_passthrough() {
        let reason = evaluate_expect_skip("python3 -c 'print(\"hello\")'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn python_script_passthrough() {
        let reason = evaluate_expect_skip("python3 /tmp/script.py");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn python_module_passthrough() {
        let reason = evaluate_expect_skip("python3 -m http.server 8080");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn long_non_python_passthrough() {
        let long_bash = format!("bash -c 'echo {}'", "x".repeat(1100));
        let reason = evaluate_expect_skip(&long_bash);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn ls_passthrough() {
        // ls is Allow via safe_rules
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn boundary_exactly_20_lines_passthrough() {
        let reason = evaluate_expect_skip(&make_heredoc(18));
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn boundary_21_lines_denied() {
        let outcome = evaluate_expect_outcome(&make_heredoc(19));
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn boundary_exactly_1000_chars_passthrough() {
        let reason = evaluate_expect_skip(&make_long_c(978));
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn boundary_1001_chars_denied() {
        let outcome = evaluate_expect_outcome(&make_long_c(979));
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python_u_flag_long_heredoc() {
        use std::fmt::Write;
        let mut cmd = "python3 -u << 'EOF'".to_owned();
        for i in 1..=25 {
            write!(cmd, "\nprint('line {i}')").expect("write to String should not fail");
        }
        cmd.push_str("\nEOF");
        let outcome = evaluate_expect_outcome(&cmd);
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
