//! One level of a parsed command hierarchy from [`CommandParser`].

use crate::prelude::*;

/// One level of a parsed command hierarchy.
///
/// A [`ParsedCommand`] contains a flat list of [`ParsedSubcommand`] where
/// index 0 is the utility, index 1 is the first subcommand, etc.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedSubcommand {
    /// Command or subcommand name.
    pub name: String,
    /// Options parsed at this command level.
    pub options: Vec<ParsedOption>,
    /// Positional arguments at this command level.
    pub operands: Vec<String>,
    /// Whether the `--` separator was encountered at this level.
    pub has_separator: bool,
}
