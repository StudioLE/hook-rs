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
    let settings = Settings::load();
    match input.tool_name.as_str() {
        "Bash" => evaluate_bash(&settings, input.tool_input.command.as_deref()?),
        "Read" => evaluate_read(&settings, input.tool_input.file_path.as_deref()?),
        _ => None,
    }
}

fn evaluate_bash(settings: &Settings, command: &str) -> Option<Outcome> {
    match Evaluator::new(settings.clone()).evaluate_str(command) {
        Ok(outcome) => Some(outcome),
        Err(report) => match report.current_context() {
            ParseError::Skip(_) => None,
            _ => Some(Outcome::ask(format!(
                "An error occurred while parsing the command: {report:?}"
            ))),
        },
    }
}
