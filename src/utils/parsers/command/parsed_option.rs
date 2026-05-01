//! Parsed option from [`CommandParser`].

/// Parsed option with its optional value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedOption {
    /// Flag name as it appeared in the input.
    pub name: String,
    /// Value if this option takes one.
    pub value: Option<String>,
}
