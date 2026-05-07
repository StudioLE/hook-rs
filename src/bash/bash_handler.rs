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

    fn run(&self, input: Self::Input) -> Option<Outcome> {
        trace!(command = %input.command, "Handling bash command");
        match self.evaluator.evaluate_str(&input.command) {
            Ok(outcome) => Some(outcome),
            Err(report) => {
                if let ParseError::Skip(reason) = report.current_context() {
                    debug!(%reason, "Skipped");
                    None
                } else {
                    error!("{}", report.render());
                    Some(Outcome::error(report))
                }
            }
        }
    }
}
