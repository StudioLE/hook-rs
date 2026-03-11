//! Top-level entry point for the hook binary.

use crate::prelude::*;

/// Read hook input from stdin, evaluate it, and print the result.
pub fn run() {
    if let Some(outcome) = evaluate_stdin() {
        outcome.print_hook_output();
    }
}

fn evaluate_stdin() -> Option<Outcome> {
    let input = match HookInput::from_stdin() {
        Ok(i) => i,
        Err(e) => {
            return Some(Outcome::ask(format!(
                "An error occurred while evaluating the hook error: {e:?}"
            )));
        }
    };
    match Evaluator::default().evaluate_str(&input.tool_input.command) {
        Ok(Ok(outcome)) => Some(outcome),
        Ok(Err(_reason)) => None,
        Err(e) => Some(Outcome::ask(format!(
            "An error occurred while parsing the command: {e:?}"
        ))),
    }
}
