//! [`Handler`] implementation for Bash tool calls.

use crate::prelude::*;

/// Evaluate Bash tool calls by parsing and matching against security rules.
#[derive(FromServices)]
pub struct BashHandler {
    /// Rule evaluator for bash commands.
    evaluator: Arc<BashEvaluator>,
}

impl Handler for BashHandler {
    type Input = BashInput;

    fn run(&self, input: HookInput<Self::Input>) -> Option<Outcome> {
        let command = input.tool_input.command;
        trace!(command, "Handling bash command");
        match self.evaluator.evaluate_str(&command, input.cwd) {
            Ok(outcome) => Some(outcome),
            Err(report) => match report.current_context() {
                ParseError::Skip(reason) => {
                    debug!(%reason, "Skipped");
                    None
                }
                ParseError::Deny(reason) => {
                    debug!(%reason, "Denied");
                    Some(Outcome::deny(reason.to_string()))
                }
                _ => {
                    error!("{}", report.render());
                    Some(Outcome::error(report))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    /// `$'…'` hides `-i` from the `sed -i` rule.
    #[test]
    fn bash_handler_run_ansi_c_quote() {
        let outcome = run("sed $'-i' s/a/b/ f.txt");
        assert_yaml_snapshot!(outcome);
    }

    /// `$'…'` hex escape decodes to `-i`.
    #[test]
    fn bash_handler_run_ansi_c_quote_hex_escape() {
        let outcome = run(r"sed $'\x2di' s/a/b/ f.txt");
        assert_eq!(outcome.expect("should match").decision, Decision::Deny);
    }

    /// `$"…"` hides `-i` from the `sed -i` rule.
    #[test]
    fn bash_handler_run_locale_quote() {
        let outcome = run(r#"sed $"-i" s/a/b/ f.txt"#);
        assert_eq!(outcome.expect("should match").decision, Decision::Deny);
    }

    /// `$'…'` is denied even when it hides nothing.
    #[test]
    fn bash_handler_run_ansi_c_quote_harmless() {
        let outcome = run(r"echo $'a\tb'");
        assert_eq!(outcome.expect("should match").decision, Decision::Deny);
    }

    /// `$'…'` inside a command substitution.
    #[test]
    fn bash_handler_run_ansi_c_quote_in_substitution() {
        let outcome = run("echo $(sed $'-i' s/a/b/ f.txt)");
        assert_eq!(outcome.expect("should match").decision, Decision::Deny);
    }

    /// Trailing `$` before a closing single quote is a regex anchor.
    #[test]
    fn bash_handler_run_single_quoted_trailing_dollar() {
        let outcome = run("rg --no-ignore '^fn main$' src");
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    /// Trailing `$` before a closing double quote is a regex anchor.
    #[test]
    fn bash_handler_run_double_quoted_trailing_dollar() {
        let outcome = run(r#"grep "end$" file"#);
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    /// Run `command` through a mock [`BashHandler`].
    fn run(command: &str) -> Option<Outcome> {
        let handler = ServiceBuilder::mock()
            .build()
            .expect_init()
            .expect::<BashHandler>();
        handler.run(HookInput::new(BashInput {
            command: command.to_owned(),
        }))
    }
}
