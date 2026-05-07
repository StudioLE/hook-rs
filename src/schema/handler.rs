//! Tool handler trait and generic dispatch.

use crate::prelude::*;

/// Tool-specific hook handler.
pub trait Handler: Send + Sync + 'static {
    /// Deserialized tool input type.
    type Input: DeserializeOwned;

    /// Evaluate the tool input against rules, returning an outcome if a rule matches.
    fn run(&self, input: Self::Input) -> Option<Outcome>;
}

/// Dispatch a tool call to the appropriate [`Handler`] via the service container.
pub trait Dispatch {
    /// Resolve a handler from the service container, deserialize input, and dispatch.
    fn dispatch<T>(&self) -> Option<Outcome>
    where
        T: Handler;
}

impl Dispatch for ServiceProvider {
    fn dispatch<T>(&self) -> Option<Outcome>
    where
        T: Handler,
    {
        let handler: Arc<T> = match self.get::<T>() {
            Ok(handler) => handler,
            Err(report) => {
                error!("{}", report.render());
                return Some(Outcome::error(report));
            }
        };
        let input = match HookInput::<T::Input>::from_stdin() {
            Ok(input) => input,
            Err(report) => {
                error!("{}", report.render());
                return Some(Outcome::error(report));
            }
        };
        handler.run(input.tool_input)
    }
}
