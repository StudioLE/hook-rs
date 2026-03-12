//! Reasons a command was skipped during parsing or evaluation.

use crate::prelude::*;

/// Reason a command was not parsed into a [`CompleteContext`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkipReason {
    /// Input contained no complete commands.
    ZeroCommands,
    /// Input contained multiple complete commands.
    MultipleCommands,
    /// Complete command contained no statements.
    ZeroStatements,
    /// Pipeline contained an unsupported compound command (while, if, subshell, brace group).
    UnsupportedCompound,
    /// A `for` loop was nested inside another `for` loop.
    NestedForLoop,
    /// All simple commands must have at least one outcome.
    OnlyAllowAll,
    /// There were no matching rules.
    NoMatches,
    /// Command had no name, only variable assignments like `FOO=bar`.
    BareAssignment,
    /// Command has an unsafe redirect (write to file, input redirect, etc.).
    UnsafeRedirect,
    /// A `for` loop has a redirect on the loop itself.
    ForLoopRedirect,
    /// A `for` loop's word list contains a command substitution.
    ForLoopSubstitution,
    /// The command name itself is a command substitution.
    CommandNameSubstitution,
    /// A command substitution is nested inside another substitution.
    NestedSubstitution,
    /// A parameter expansion contains a command substitution.
    ParameterSubstitution,
    /// An arithmetic expression could contain a command substitution.
    ArithmeticSubstitution,
    /// Command has a process substitution (`<(...)` or `>(...)`).
    ProcessSubstitution,
}
