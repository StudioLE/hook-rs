use claude_hooks::evaluate::evaluate;
use claude_hooks::prelude::*;

fn main() {
    match run() {
        Ok(Some(outcome)) => outcome.print_hook_output(),
        Ok(None) => {}
        Err(e) => Outcome::ask(format!(
            "An error occurred while evaluating the hook error: {e:?}"
        ))
        .print_hook_output(),
    }
}

fn run() -> Result<Option<Outcome>, error_stack::Report<HookError>> {
    let input = HookInput::from_stdin()?;
    Ok(evaluate(&input.tool_input.command))
}
