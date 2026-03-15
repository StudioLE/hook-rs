//! Tool handler trait and generic dispatch.

use crate::prelude::*;

/// Tool-specific hook handler.
pub trait Handler {
    /// Deserialized tool input type.
    type Input: DeserializeOwned;

    /// Evaluate the tool input against settings, returning an outcome if a rule matches.
    fn run(input: Self::Input, settings: Settings) -> Option<Outcome>;
}

/// Deserialize stdin as `T::Input` and dispatch to the handler.
pub fn run<T: Handler>() -> Option<Outcome> {
    let input = match HookInput::<T::Input>::from_stdin() {
        Ok(i) => i,
        Err(e) => {
            return Some(Outcome::ask(format!(
                "An error occurred while evaluating the hook error: {e:?}"
            )));
        }
    };
    let settings = Settings::load();
    T::run(input.tool_input, settings)
}
