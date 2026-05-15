//! Rules for Python commands.

use crate::prelude::*;

const REASON: &str = "`python` is blocked. Alternatives: shell tools, `jq`, `yq`, Rust";

const MAX_CHARS: usize = 1000;
const MAX_LINES: usize = 20;

/// Rules for all Python commands.
pub fn python_rules() -> Vec<BashRule> {
    vec![
        python(),
        python3(),
        python__long_inline(),
        python3__long_inline(),
    ]
}

/// Rule for `python` commands.
fn python() -> BashRule {
    BashRule::new("python", "python", Outcome::deny(REASON))
}

/// Rule for `python3` commands.
fn python3() -> BashRule {
    BashRule::new("python3", "python3", Outcome::deny(REASON))
}

/// Deny excessively long inline `python` commands.
fn python__long_inline() -> BashRule {
    BashRule {
        condition: Some(is_long_inline),
        ..BashRule::new(
            "python__long_inline",
            "python",
            Outcome::deny(format!(
                "Inline `python` over {MAX_CHARS} chars or {MAX_LINES} lines is blocked. Alternatives: write a script to `/tmp/` and run it"
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
                "Inline `python` over {MAX_CHARS} chars or {MAX_LINES} lines is blocked. Alternatives: write a script to `/tmp/` and run it"
            )),
        )
    }
}

fn is_long_inline(ctx: &BashRuleContext) -> bool {
    let has_inline = ctx.simple.args.iter().any(|a| a == "-c") || ctx.simple.has_heredoc;
    has_inline
        && (ctx.complete.raw.len() > MAX_CHARS || ctx.complete.raw.lines().count() > MAX_LINES)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn python_inline() {
        let result = eval_rules(python_rules(), "python -c 'print(1)'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_inline() {
        let result = eval_rules(python_rules(), "python3 -c 'print(1)'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_script() {
        let result = eval_rules(python_rules(), "python3 /tmp/script.py");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_module() {
        let result = eval_rules(python_rules(), "python3 -m http.server 8080");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_heredoc() {
        let result = eval_rules(python_rules(), "python3 << 'EOF'\nprint('hello')\nEOF");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn python3_bare() {
        let result = eval_rules(python_rules(), "python3");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
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
        let result = eval_rules(python_rules(), &make_heredoc(25));
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn long_heredoc_python2() {
        let cmd = make_heredoc(25).replace("python3", "python");
        let result = eval_rules(python_rules(), &cmd);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn long_c_1001_chars() {
        let result = eval_rules(python_rules(), &make_long_c(979));
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn short_heredoc() {
        let result = eval_rules(python_rules(), &make_heredoc(5));
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn short_c() {
        let result = eval_rules(python_rules(), "python3 -c 'print(\"hello\")'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn long_non_python() {
        let long_bash = format!("bash -c 'echo {}'", "x".repeat(1100));
        let result = eval_rules(python_rules(), &long_bash);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn ls() {
        let result = eval_rules(python_rules(), "ls -la");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn boundary_exactly_20_lines() {
        let result = eval_rules(python_rules(), &make_heredoc(18));
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn boundary_21_lines() {
        let result = eval_rules(python_rules(), &make_heredoc(19));
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn boundary_exactly_1000_chars() {
        let result = eval_rules(python_rules(), &make_long_c(978));
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn boundary_1001_chars() {
        let result = eval_rules(python_rules(), &make_long_c(979));
        let outcome = expect_outcome(result);
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
        let result = eval_rules(python_rules(), &cmd);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
