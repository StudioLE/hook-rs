//! Compound structure nesting indicator.

use crate::prelude::*;

/// Compound structure that a command can be nested inside.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Nesting {
    /// Inside a `for` loop body.
    For,
    /// Inside a command substitution.
    Substitution,
}
