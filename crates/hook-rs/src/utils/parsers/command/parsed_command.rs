//! Parsed command output from [`CommandParser`].

use crate::prelude::*;

/// Parsed command output from [`CommandParser`].
///
/// Wraps the flat list of [`ParsedSubcommand`] levels produced by parsing.
/// Index 0 is the utility, index 1 is the first subcommand, etc.
///
/// Derefs to `Vec<ParsedSubcommand>` for direct access to levels.
#[derive(Debug)]
pub struct ParsedCommand {
    inner: Vec<ParsedSubcommand>,
}

impl ParsedCommand {
    /// Create a new [`ParsedCommand`] from a list of [`ParsedSubcommand`] levels.
    pub fn new(inner: Vec<ParsedSubcommand>) -> Self {
        Self { inner }
    }
}

impl Deref for ParsedCommand {
    type Target = Vec<ParsedSubcommand>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deref provides direct access to [`ParsedSubcommand`] levels.
    #[test]
    fn parsed_command_deref() {
        // Arrange
        let command = ParsedCommand::new(vec![
            ParsedSubcommand {
                name: "git".to_owned(),
                options: Vec::new(),
                operands: Vec::new(),
                has_separator: false,
            },
            ParsedSubcommand {
                name: "log".to_owned(),
                options: Vec::new(),
                operands: vec!["src/main.rs".to_owned()],
                has_separator: false,
            },
        ]);

        // Assert
        assert_eq!(command.len(), 2);
        assert_eq!(command.first().expect("has root").name, "git");
        assert_eq!(
            command.get(1).expect("has subcommand").operands,
            vec!["src/main.rs"]
        );
    }
}
