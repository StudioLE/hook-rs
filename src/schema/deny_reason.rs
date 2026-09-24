//! Reasons a command was denied during parsing.

use crate::prelude::*;

/// Reason a command was denied before rule evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum DenyReason {
    /// Command uses `$'…'` or `$"…"` quoting, which rules cannot see through.
    #[error("`$'…'` and `$\"…\"` quoting is blocked. Use plain single or double quotes")]
    DollarQuote,
}
