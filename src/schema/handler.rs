//! Tool handler trait.

use crate::prelude::*;

/// Tool-specific hook handler.
pub trait Handler: Send + Sync + 'static {
    /// Deserialized tool input type.
    type Input: DeserializeOwned;

    /// Evaluate the tool input against rules, returning an outcome if a rule matches.
    fn run(&self, input: Self::Input) -> Option<Outcome>;
}
