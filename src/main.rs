use std::io::{self, Read};

use claude_hooks::evaluate::evaluate;
use claude_hooks::types::{CheckResult, HookInput, HookOutput};

fn main() {
    match run() {
        Ok(Some(result)) => print_output(result),
        Ok(None) => {}
        Err(e) => print_output(CheckResult::deny(format!("Hook error: {e:?}"))),
    }
}

fn print_output(result: CheckResult) {
    let output = HookOutput::from(result);
    let json = serde_json::to_string(&output).expect("serialization should not fail");
    println!("{json}");
}

fn run() -> Result<Option<CheckResult>, error_stack::Report<HookError>> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| error_stack::Report::new(HookError::Io(e)))?;
    let hook_input: HookInput =
        serde_json::from_str(&input).map_err(|e| error_stack::Report::new(HookError::Json(e)))?;
    Ok(evaluate(&hook_input.tool_input.command))
}

#[derive(Debug, thiserror::Error)]
enum HookError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("JSON parse error")]
    Json(#[from] serde_json::Error),
}
