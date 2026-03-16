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
        Ok(input) => input,
        Err(report) => {
            error!("{report:?}");
            return Some(Outcome::error(report));
        }
    };
    let settings = match Settings::load() {
        Ok(settings) => settings,
        Err(report) => {
            error!("{report:?}");
            return Some(Outcome::error(report));
        }
    };
    T::run(input.tool_input, settings)
}
