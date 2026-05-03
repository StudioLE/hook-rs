//! Parsed command output from [`CommandParser`].

use crate::prelude::*;

/// Parsed command output from [`CommandParser`].
///
/// Wraps the flat list of [`Subcommand`] levels produced by parsing.
/// Index 0 is the utility, index 1 is the first subcommand, etc.
///
/// Derefs to `Vec<Subcommand>` for direct access to levels.
#[derive(Debug)]
pub struct ParsedCommand {
    inner: Vec<Subcommand>,
}

impl ParsedCommand {
    /// Create a new [`ParsedCommand`] from a list of [`Subcommand`] levels.
    pub fn new(inner: Vec<Subcommand>) -> Self {
        Self { inner }
    }
}

impl Deref for ParsedCommand {
    type Target = Vec<Subcommand>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deref provides direct access to [`Subcommand`] levels.
    #[test]
    fn parsed_command_deref() {
        // Arrange
        let command = ParsedCommand::new(vec![
            Subcommand {
                name: "git".to_owned(),
                options: Vec::new(),
                operands: Vec::new(),
                has_separator: false,
            },
            Subcommand {
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
