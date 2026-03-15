//! [`Handler`] implementation for Bash tool calls.

use crate::prelude::*;

/// Evaluate Bash tool calls by parsing and matching against security rules.
pub struct BashHandler;

impl Handler for BashHandler {
    type Input = BashInput;

    fn run(input: Self::Input, settings: Settings) -> Option<Outcome> {
        match BashEvaluator::new(settings).evaluate_str(&input.command) {
            Ok(outcome) => Some(outcome),
            Err(report) => match report.current_context() {
                ParseError::Skip(_) => None,
                _ => Some(Outcome::ask(format!(
                    "An error occurred while parsing the command: {report:?}"
                ))),
            },
        }
    }
}
