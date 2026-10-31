//! Recursive command definition for hierarchical argument parsing.

use crate::prelude::*;

/// Define a command or subcommand with its options, subcommands, and operands.
///
/// Each node represents one level in the command hierarchy. The tree is
/// recursive: subcommands are themselves `CommandSchema` nodes.
#[derive(Clone, Debug)]
pub struct CommandSchema {
    /// Command or subcommand name.
    pub name: String,
    /// Options valid at this command level.
    pub options: Vec<OptionSchema>,
    /// Subcommands available at this command level.
    pub subcommands: Vec<CommandSchema>,
    /// Positional argument slots after all options and subcommands.
    pub operands: Vec<OperandSchema>,
}

impl CommandSchema {
    /// Find an option definition matching the given flag name.
    pub fn find_option(&self, flag: &str) -> Option<&OptionSchema> {
        self.options.iter().find(|o| o.matches(flag))
    }

    /// Find a subcommand definition matching the given name.
    pub fn find_subcommand(&self, name: &str) -> Option<&CommandSchema> {
        self.subcommands.iter().find(|s| s.name == name)
    }
}
