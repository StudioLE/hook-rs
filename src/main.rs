use std::io::{self, Read};

use error_stack::ResultExt;

use claude_hooks::check::CheckResult;
use claude_hooks::evaluate::evaluate;
use claude_hooks::hook::{HookInput, HookOutput};

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
        .change_context(HookError::ReadStdin)?;
    let hook_input: HookInput =
        serde_json::from_str(&input).change_context(HookError::DeserializeInput)?;
    Ok(evaluate(&hook_input.tool_input.command))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum HookError {
    #[error("read stdin")]
    ReadStdin,
    #[error("deserialize hook input")]
    DeserializeInput,
}
