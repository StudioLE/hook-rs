//! Builder for [`CommandSchema`].

use crate::prelude::*;

/// Build a [`CommandSchema`] with sensible defaults.
pub struct CommandSchemaBuilder {
    name: String,
    options: Vec<OptionSchema>,
    subcommands: Vec<CommandSchema>,
    operands: Vec<OperandSchema>,
}

impl CommandSchemaBuilder {
    /// Create a new builder with the given command name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            options: Vec::new(),
            subcommands: Vec::new(),
            operands: Vec::new(),
        }
    }

    /// Add an option to this command level.
    pub fn with_option(mut self, option: OptionSchema) -> Self {
        self.options.push(option);
        self
    }

    /// Add a subcommand to this command level.
    pub fn with_subcommand(mut self, subcommand: CommandSchema) -> Self {
        self.subcommands.push(subcommand);
        self
    }

    /// Add an operand to this command level.
    pub fn with_operand(mut self, operand: OperandSchema) -> Self {
        self.operands.push(operand);
        self
    }

    /// Build the [`CommandSchema`].
    pub fn build(self) -> CommandSchema {
        CommandSchema {
            name: self.name,
            options: self.options,
            subcommands: self.subcommands,
            operands: self.operands,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "test assertions: panic is the intended failure mode"
)]
mod tests {
    use super::*;

    #[test]
    fn command_schema_builder_minimal() {
        // Act
        let output = CommandSchemaBuilder::new("git").build();

        // Assert
        assert_eq!(output.name, "git");
        assert!(output.options.is_empty());
        assert!(output.subcommands.is_empty());
        assert!(output.operands.is_empty());
    }

    /// Full docker compose schema exercises nested builders.
    #[test]
    fn command_schema_builder_nested() {
        // Arrange & Act
        let output = CommandSchemaBuilder::new("docker")
            .with_subcommand(
                CommandSchemaBuilder::new("compose")
                    .with_option(
                        OptionSchemaBuilder::new(["-f", "--file"])
                            .with_value(ValueConstraint::Any)
                            .build(),
                    )
                    .with_subcommand(
                        CommandSchemaBuilder::new("run")
                            .with_option(OptionSchemaBuilder::new(["--build"]).build())
                            .with_option(OptionSchemaBuilder::new(["-v"]).with_repeatable().build())
                            .with_operand(OperandSchemaBuilder::new("service").build())
                            .with_operand(OperandSchemaBuilder::new("args").with_variadic().build())
                            .build(),
                    )
                    .build(),
            )
            .build();

        // Assert
        assert_eq!(output.name, "docker");
        assert_eq!(output.subcommands.len(), 1);
        let compose = &output.subcommands[0];
        assert_eq!(compose.name, "compose");
        assert_eq!(compose.options.len(), 1);
        assert_eq!(compose.subcommands.len(), 1);
        let run = &compose.subcommands[0];
        assert_eq!(run.name, "run");
        assert_eq!(run.options.len(), 2);
        assert_eq!(run.operands.len(), 2);
        assert!(run.operands[1].variadic);
    }
}
