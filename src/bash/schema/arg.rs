//! Classified command-line argument from [`ArgParser`].

/// Classified command-line argument.
///
/// Each variant represents one element of a parsed argument list,
/// tagged with its role as determined by an [`ArgSchema`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Arg {
    /// Boolean flag with no associated value.
    Flag(String),
    /// Flag paired with its value.
    FlagPair(String, String),
    /// Non-flag argument.
    Operand(String),
    /// End-of-options separator (`--`).
    Separator,
}
