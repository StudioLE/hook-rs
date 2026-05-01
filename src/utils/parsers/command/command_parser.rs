//! Schema-aware hierarchical command parser.

use crate::prelude::*;

/// Schema-aware hierarchical command parser.
///
/// Holds a [`CommandSchema`] and parses token streams against it,
/// producing `Vec<ParsedCommand>` output.
///
/// Strips shell quotes from tokens by default via [`unquote_str`].
/// Disable with [`CommandParser::without_unquote`].
pub struct CommandParser {
    schema: CommandSchema,
    unquote: bool,
}

impl CommandParser {
    /// Create a new parser for the given schema.
    ///
    /// Default: unquoting enabled.
    pub fn new(schema: CommandSchema) -> Self {
        Self {
            schema,
            unquote: true,
        }
    }

    /// Disable shell-quote stripping from tokens.
    pub fn without_unquote(mut self) -> Self {
        self.unquote = false;
        self
    }

    /// Parse tokens against the schema.
    pub fn parse(
        &self,
        tokens: Vec<String>,
    ) -> Result<Vec<ParsedCommand>, Report<CommandParseError>> {
        let tokens = if self.unquote {
            tokens.into_iter().map(|t| unquote_str(&t)).collect()
        } else {
            tokens
        };
        let state = ParserState {
            queue: VecDeque::from(tokens),
            output: Vec::new(),
            current_options: Vec::new(),
            current_operands: Vec::new(),
            after_separator: false,
            current_schema: self.schema.clone(),
        };
        state.run()
    }
}

struct ParserState {
    queue: VecDeque<String>,
    output: Vec<ParsedCommand>,
    current_options: Vec<ParsedOption>,
    current_operands: Vec<String>,
    after_separator: bool,
    current_schema: CommandSchema,
}

impl ParserState {
    fn run(mut self) -> Result<Vec<ParsedCommand>, Report<CommandParseError>> {
        while self.peek().is_some() {
            if self.after_separator {
                let token = self.next().expect("peek confirmed token");
                self.current_operands.push(token);
                continue;
            }
            if self.peek_is("--") {
                self.next();
                self.after_separator = true;
                continue;
            }
            if self.peek_starts_with("--") {
                self.parse_long_flag()?;
                continue;
            }
            if self.peek_is_short_flag() {
                let raw = self.next().expect("peek confirmed token");
                self.parse_short_flags(raw)?;
                continue;
            }
            let token = self.peek().expect("peek confirmed token").to_owned();
            if let Some(child) = self.current_schema.find_subcommand(&token).cloned() {
                self.flush_level()?;
                self.current_schema = child;
                self.next();
                continue;
            }
            let token = self.next().expect("peek confirmed token");
            self.current_operands.push(token);
        }
        self.flush_level()?;
        Ok(self.output)
    }

    fn next(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    fn peek(&self) -> Option<&str> {
        self.queue.front().map(String::as_str)
    }

    fn peek_is(&self, value: &str) -> bool {
        self.peek() == Some(value)
    }

    fn peek_starts_with(&self, prefix: &str) -> bool {
        self.peek()
            .is_some_and(|s| s.starts_with(prefix) && s.len() > prefix.len())
    }

    fn peek_is_short_flag(&self) -> bool {
        self.peek()
            .is_some_and(|s| s.starts_with('-') && s.len() >= 2 && !s.starts_with("--"))
    }

    /// Emit a [`ParsedCommand`] for the current schema level and reset state.
    fn flush_level(&mut self) -> Result<(), Report<CommandParseError>> {
        self.validate_level()?;
        self.output.push(ParsedCommand {
            name: self.current_schema.name.clone(),
            options: take(&mut self.current_options),
            operands: take(&mut self.current_operands),
            has_separator: self.after_separator,
        });
        self.after_separator = false;
        Ok(())
    }

    fn parse_long_flag(&mut self) -> Result<(), Report<CommandParseError>> {
        let raw = self.next().expect("peek confirmed token");
        if let Some((flag, value)) = raw.split_once('=') {
            let schema = self
                .current_schema
                .find_option(flag)
                .ok_or_else(|| flag_error(CommandParseError::UnknownFlag, &raw))?;
            let constraint = schema
                .value
                .as_ref()
                .ok_or_else(|| flag_error(CommandParseError::UnknownFlag, &raw))?;
            validate_option_value(flag, value, constraint)?;
            self.current_options.push(ParsedOption {
                name: String::from(flag),
                value: Some(String::from(value)),
            });
            return Ok(());
        }
        let schema = self
            .current_schema
            .find_option(&raw)
            .ok_or_else(|| flag_error(CommandParseError::UnknownFlag, &raw))?;
        let constraint = schema.value.clone();
        if let Some(constraint) = &constraint {
            let value = self
                .next()
                .ok_or_else(|| flag_error(CommandParseError::MissingValue, &raw))?;
            validate_option_value(&raw, &value, constraint)?;
            self.current_options.push(ParsedOption {
                name: raw,
                value: Some(value),
            });
        } else {
            self.current_options.push(ParsedOption {
                name: raw,
                value: None,
            });
        }
        Ok(())
    }

    fn parse_short_flags(&mut self, raw: String) -> Result<(), Report<CommandParseError>> {
        let chars: Vec<char> = raw.chars().skip(1).collect();
        for (i, &c) in chars.iter().enumerate() {
            let flag = format!("-{c}");
            let schema = self
                .current_schema
                .find_option(&flag)
                .ok_or_else(|| flag_error(CommandParseError::UnknownFlag, &flag))?;
            let constraint = schema.value.clone();
            if let Some(constraint) = &constraint {
                let remainder: String = chars.get(i + 1..).unwrap_or_default().iter().collect();
                let value = if remainder.is_empty() {
                    self.next()
                        .ok_or_else(|| flag_error(CommandParseError::MissingValue, &flag))?
                } else {
                    remainder
                };
                validate_option_value(&flag, &value, constraint)?;
                self.current_options.push(ParsedOption {
                    name: flag,
                    value: Some(value),
                });
                return Ok(());
            }
            self.current_options.push(ParsedOption {
                name: flag,
                value: None,
            });
        }
        Ok(())
    }

    /// Validate current operands against the schema's operand constraints.
    fn validate_level(&self) -> Result<(), Report<CommandParseError>> {
        let schemas = &self.current_schema.operands;
        if schemas.is_empty() {
            return Ok(());
        }
        let has_variadic = schemas.last().is_some_and(|s| s.variadic);
        let min_required = schemas
            .iter()
            .filter(|s| !s.optional && !s.variadic)
            .count();
        let max_allowed = if has_variadic {
            usize::MAX
        } else {
            schemas.len()
        };
        if self.current_operands.len() < min_required {
            return Err(Report::new(CommandParseError::TooFewOperands)
                .attach("command", &self.current_schema.name)
                .attach("expected", format!("at least {min_required}"))
                .attach("actual", self.current_operands.len().to_string()));
        }
        if self.current_operands.len() > max_allowed {
            return Err(Report::new(CommandParseError::TooManyOperands)
                .attach("command", &self.current_schema.name)
                .attach("expected", format!("at most {max_allowed}"))
                .attach("actual", self.current_operands.len().to_string()));
        }
        for (i, operand) in self.current_operands.iter().enumerate() {
            let schema_index = i.min(schemas.len() - 1);
            let schema = schemas
                .get(schema_index)
                .expect("index bounded by min with non-empty schemas");
            validate_operand_value(&schema.name, operand, &schema.value)?;
        }
        Ok(())
    }
}

fn validate_option_value(
    flag: &str,
    value: &str,
    constraint: &ValueConstraint,
) -> Result<(), Report<CommandParseError>> {
    if constraint.matches(value) {
        Ok(())
    } else {
        Err(Report::new(CommandParseError::InvalidFlagValue)
            .attach("flag", flag)
            .attach("value", value)
            .attach("expected", constraint.description()))
    }
}

fn validate_operand_value(
    name: &str,
    value: &str,
    constraint: &ValueConstraint,
) -> Result<(), Report<CommandParseError>> {
    if constraint.matches(value) {
        Ok(())
    } else {
        Err(Report::new(CommandParseError::InvalidOperandValue)
            .attach("operand", name)
            .attach("value", value)
            .attach("expected", constraint.description()))
    }
}

fn flag_error(error: CommandParseError, flag: &str) -> Report<CommandParseError> {
    Report::new(error).attach("flag", flag)
}

/// Errors returned by [`CommandParser::parse`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CommandParseError {
    /// Flag not present in the schema.
    #[error("Unknown flag")]
    UnknownFlag,
    /// Value flag at end of args with no following value.
    #[error("Missing value for flag")]
    MissingValue,
    /// Value does not match the option's constraint.
    #[error("Invalid flag value")]
    InvalidFlagValue,
    /// Fewer operands than the schema requires.
    #[error("Too few operands")]
    TooFewOperands,
    /// More operands than the schema accepts.
    #[error("Too many operands")]
    TooManyOperands,
    /// Operand value does not match the schema's constraint.
    #[error("Invalid operand value")]
    InvalidOperandValue,
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "test assertions: panic is the intended failure mode"
)]
mod tests {
    use super::*;

    /// Simple schema: git log --oneline -n <num>
    fn git_log_schema() -> CommandSchema {
        CommandSchemaBuilder::new("git")
            .with_subcommand(
                CommandSchemaBuilder::new("log")
                    .with_option(OptionSchemaBuilder::new(["--oneline"]).build())
                    .with_option(
                        OptionSchemaBuilder::new(["-n"])
                            .with_value(ValueConstraint::Any)
                            .build(),
                    )
                    .with_operand(OperandSchemaBuilder::new("path").with_variadic().build())
                    .build(),
            )
            .build()
    }

    fn parse(
        schema: CommandSchema,
        input: &[&str],
    ) -> Result<Vec<ParsedCommand>, Report<CommandParseError>> {
        let tokens: Vec<String> = input.iter().map(|s| String::from(*s)).collect();
        CommandParser::new(schema).parse(tokens)
    }

    #[test]
    fn command_parser_subcommand_with_flags() {
        // Arrange
        let schema = git_log_schema();

        // Act
        let output = parse(schema, &["log", "--oneline", "-n", "5"]).expect("should parse");

        // Assert
        assert_eq!(output.len(), 2);
        assert_eq!(output[0].name, "git");
        assert!(output[0].options.is_empty());
        assert_eq!(output[1].name, "log");
        assert_eq!(output[1].options.len(), 2);
        assert_eq!(output[1].options[0].name, "--oneline");
        assert_eq!(output[1].options[0].value, None);
        assert_eq!(output[1].options[1].name, "-n");
        assert_eq!(output[1].options[1].value, Some(String::from("5")));
    }

    #[test]
    fn command_parser_operands() {
        // Arrange
        let schema = git_log_schema();

        // Act
        let output = parse(schema, &["log", "src/main.rs"]).expect("should parse");

        // Assert
        assert_eq!(output[1].operands, vec![String::from("src/main.rs")]);
    }

    #[test]
    fn command_parser_separator() {
        // Arrange
        let schema = git_log_schema();

        // Act
        let output =
            parse(schema, &["log", "--oneline", "--", "--weird-file"]).expect("should parse");

        // Assert
        assert!(output[1].has_separator);
        assert_eq!(output[1].operands, vec![String::from("--weird-file")]);
        assert_eq!(output[1].options.len(), 1);
    }

    #[test]
    fn command_parser_unknown_flag() {
        // Arrange
        let schema = git_log_schema();

        // Act
        let error = parse(schema, &["log", "--unknown"]).expect_err("should fail");

        // Assert
        assert_eq!(*error.current_context(), CommandParseError::UnknownFlag);
    }

    #[test]
    fn command_parser_missing_value() {
        // Arrange
        let schema = git_log_schema();

        // Act
        let error = parse(schema, &["log", "-n"]).expect_err("should fail");

        // Assert
        assert_eq!(*error.current_context(), CommandParseError::MissingValue);
    }

    #[test]
    fn command_parser_nested_subcommands() {
        // Arrange
        let schema = CommandSchemaBuilder::new("docker")
            .with_subcommand(
                CommandSchemaBuilder::new("compose")
                    .with_option(
                        OptionSchemaBuilder::new(["-f"])
                            .with_value(ValueConstraint::Any)
                            .build(),
                    )
                    .with_subcommand(
                        CommandSchemaBuilder::new("run")
                            .with_option(OptionSchemaBuilder::new(["--build"]).build())
                            .with_operand(OperandSchemaBuilder::new("service").build())
                            .build(),
                    )
                    .build(),
            )
            .build();

        // Act
        let output = parse(
            schema,
            &["compose", "-f", "compose.yml", "run", "--build", "web"],
        )
        .expect("should parse");

        // Assert
        assert_eq!(output.len(), 3);
        assert_eq!(output[0].name, "docker");
        assert_eq!(output[1].name, "compose");
        assert_eq!(output[1].options.len(), 1);
        assert_eq!(output[1].options[0].name, "-f");
        assert_eq!(
            output[1].options[0].value,
            Some(String::from("compose.yml"))
        );
        assert_eq!(output[2].name, "run");
        assert_eq!(output[2].options.len(), 1);
        assert_eq!(output[2].options[0].name, "--build");
        assert_eq!(output[2].operands, vec![String::from("web")]);
    }

    #[test]
    fn command_parser_separator_resets_at_subcommand() {
        // Arrange
        let schema = CommandSchemaBuilder::new("tool")
            .with_subcommand(
                CommandSchemaBuilder::new("sub")
                    .with_option(OptionSchemaBuilder::new(["--flag"]).build())
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["--", "sub", "--flag"]).expect("should parse");

        // Assert
        assert!(output[0].has_separator);
        assert_eq!(
            output[0].operands,
            vec![String::from("sub"), String::from("--flag")]
        );
        assert_eq!(output.len(), 1);
    }

    #[test]
    fn command_parser_bundled_short_flags() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(OptionSchemaBuilder::new(["-a"]).build())
            .with_option(OptionSchemaBuilder::new(["-b"]).build())
            .with_option(
                OptionSchemaBuilder::new(["-n"])
                    .with_value(ValueConstraint::Any)
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["-abn5"]).expect("should parse");

        // Assert
        assert_eq!(output[0].options.len(), 3);
        assert_eq!(output[0].options[0].name, "-a");
        assert_eq!(output[0].options[1].name, "-b");
        assert_eq!(output[0].options[2].name, "-n");
        assert_eq!(output[0].options[2].value, Some(String::from("5")));
    }

    #[test]
    fn command_parser_long_flag_equals() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["--format"])
                    .with_value(ValueConstraint::Any)
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["--format=json"]).expect("should parse");

        // Assert
        assert_eq!(output[0].options[0].name, "--format");
        assert_eq!(output[0].options[0].value, Some(String::from("json")));
    }

    #[test]
    fn command_parser_no_subcommand() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("file").build())
            .build();

        // Act
        let output = parse(schema, &["myfile.txt"]).expect("should parse");

        // Assert
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].operands, vec![String::from("myfile.txt")]);
    }

    #[test]
    fn command_parser_anyof_option_valid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["--color"])
                    .with_value(ValueConstraint::any_of(vec![
                        String::from("always"),
                        String::from("never"),
                        String::from("auto"),
                    ]))
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["--color", "always"]).expect("should parse");

        // Assert
        assert_eq!(output[0].options[0].value, Some(String::from("always")));
    }

    #[test]
    fn command_parser_anyof_option_invalid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["--color"])
                    .with_value(ValueConstraint::any_of(vec![
                        String::from("always"),
                        String::from("never"),
                    ]))
                    .build(),
            )
            .build();

        // Act
        let error = parse(schema, &["--color", "rainbow"]).expect_err("should fail");

        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidFlagValue
        );
    }

    #[test]
    fn command_parser_glob_option_valid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["--file"])
                    .with_value(ValueConstraint::glob("*.yml").expect("valid glob"))
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["--file", "docker-compose.yml"]).expect("should parse");

        // Assert
        assert_eq!(
            output[0].options[0].value,
            Some(String::from("docker-compose.yml"))
        );
    }

    #[test]
    fn command_parser_glob_option_invalid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["--file"])
                    .with_value(ValueConstraint::glob("*.yml").expect("valid glob"))
                    .build(),
            )
            .build();

        // Act
        let error = parse(schema, &["--file", "config.toml"]).expect_err("should fail");

        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidFlagValue
        );
    }

    #[test]
    fn command_parser_regex_option_valid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["-n"])
                    .with_value(ValueConstraint::regex(r"^\d+$").expect("valid regex"))
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["-n", "42"]).expect("should parse");

        // Assert
        assert_eq!(output[0].options[0].value, Some(String::from("42")));
    }

    #[test]
    fn command_parser_regex_option_invalid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["-n"])
                    .with_value(ValueConstraint::regex(r"^\d+$").expect("valid regex"))
                    .build(),
            )
            .build();

        // Act
        let error = parse(schema, &["-n", "abc"]).expect_err("should fail");

        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidFlagValue
        );
    }

    #[test]
    fn command_parser_too_few_operands() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("required").build())
            .build();

        // Act
        let error = parse(schema, &[]).expect_err("should fail");

        // Assert
        assert_eq!(*error.current_context(), CommandParseError::TooFewOperands);
    }

    #[test]
    fn command_parser_too_many_operands() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("only-one").build())
            .build();

        // Act
        let error = parse(schema, &["a", "b"]).expect_err("should fail");

        // Assert
        assert_eq!(*error.current_context(), CommandParseError::TooManyOperands);
    }

    #[test]
    fn command_parser_variadic_operands() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("first").build())
            .with_operand(OperandSchemaBuilder::new("rest").with_variadic().build())
            .build();

        // Act
        let output = parse(schema, &["a", "b", "c"]).expect("should parse");

        // Assert
        assert_eq!(
            output[0].operands,
            vec![String::from("a"), String::from("b"), String::from("c"),]
        );
    }

    #[test]
    fn command_parser_anyof_operand_invalid() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(
                OperandSchemaBuilder::new("action")
                    .with_value(ValueConstraint::any_of(vec![
                        String::from("start"),
                        String::from("stop"),
                    ]))
                    .build(),
            )
            .build();

        // Act
        let error = parse(schema, &["explode"]).expect_err("should fail");

        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidOperandValue
        );
    }

    /// When a command has no operand schemas, any operands are accepted.
    #[test]
    fn command_parser_no_operand_schema() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd").build();

        // Act
        let output = parse(schema, &["a", "b", "c"]).expect("should parse");

        // Assert
        assert_eq!(
            output[0].operands,
            vec![String::from("a"), String::from("b"), String::from("c"),]
        );
    }

    /// Double-quoted operand is unquoted by default.
    #[test]
    fn command_parser_unquote_double_quoted_operand() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd").build();

        // Act
        let output = parse(schema, &["\"hello\""]).expect("should parse");

        // Assert
        assert_eq!(output[0].operands, vec![String::from("hello")]);
    }

    /// Single-quoted operand is unquoted by default.
    #[test]
    fn command_parser_unquote_single_quoted_operand() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd").build();

        // Act
        let output = parse(schema, &["'hello'"]).expect("should parse");

        // Assert
        assert_eq!(output[0].operands, vec![String::from("hello")]);
    }

    /// Quoted flag value is unquoted by default.
    #[test]
    fn command_parser_unquote_flag_value() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_option(
                OptionSchemaBuilder::new(["-f"])
                    .with_value(ValueConstraint::Any)
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema, &["-f", "\"value\""]).expect("should parse");

        // Assert
        assert_eq!(output[0].options[0].value, Some(String::from("value")));
    }

    /// Quoted subcommand name is unquoted and matched.
    #[test]
    fn command_parser_unquote_subcommand() {
        // Arrange
        let schema = CommandSchemaBuilder::new("git")
            .with_subcommand(CommandSchemaBuilder::new("log").build())
            .build();

        // Act
        let output = parse(schema, &["\"log\""]).expect("should parse");

        // Assert
        assert_eq!(output.len(), 2);
        assert_eq!(output[1].name, "log");
    }

    /// Quotes are preserved when unquoting is disabled.
    #[test]
    fn command_parser_without_unquote() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd").build();
        let tokens = vec![String::from("\"hello\"")];

        // Act
        let output = CommandParser::new(schema)
            .without_unquote()
            .parse(tokens)
            .expect("should parse");

        // Assert
        assert_eq!(output[0].operands, vec![String::from("\"hello\"")]);
    }

    /// Optional operand may be omitted without error.
    #[test]
    fn command_parser_optional_operand_absent() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("required").build())
            .with_operand(OperandSchemaBuilder::new("extra").with_optional().build())
            .build();

        // Act
        let output = parse(schema, &["value"]).expect("should parse");

        // Assert
        assert_eq!(output[0].operands, vec![String::from("value")]);
    }

    /// Optional operand is accepted and validated when provided.
    #[test]
    fn command_parser_optional_operand_present() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("required").build())
            .with_operand(OperandSchemaBuilder::new("extra").with_optional().build())
            .build();

        // Act
        let output = parse(schema, &["value", "bonus"]).expect("should parse");

        // Assert
        assert_eq!(
            output[0].operands,
            vec![String::from("value"), String::from("bonus")]
        );
    }

    /// Optional operand is validated against its constraint when provided.
    #[test]
    fn command_parser_optional_operand_validated() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(OperandSchemaBuilder::new("required").build())
            .with_operand(
                OperandSchemaBuilder::new("num")
                    .with_optional()
                    .with_value(ValueConstraint::regex(r"^\d+$").expect("valid regex"))
                    .build(),
            )
            .build();

        // Act
        let error = parse(schema, &["value", "abc"]).expect_err("should fail");

        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidOperandValue
        );
    }

    /// Each operand is validated against the schema at its position.
    #[test]
    fn command_parser_operand_positional_validation() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd")
            .with_operand(
                OperandSchemaBuilder::new("action")
                    .with_value(ValueConstraint::any_of(vec![
                        String::from("start"),
                        String::from("stop"),
                    ]))
                    .build(),
            )
            .with_operand(
                OperandSchemaBuilder::new("count")
                    .with_value(ValueConstraint::regex(r"^\d+$").expect("valid regex"))
                    .build(),
            )
            .build();

        // Act
        let output = parse(schema.clone(), &["start", "42"]).expect("should parse");

        // Assert
        assert_eq!(
            output[0].operands,
            vec![String::from("start"), String::from("42")]
        );
        
        // Act
        let error = parse(schema.clone(), &["explode", "42"]).expect_err("should fail");
        
        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidOperandValue
        );
        
        // Act
        let error = parse(schema, &["start", "abc"]).expect_err("should fail");
        
        // Assert
        assert_eq!(
            *error.current_context(),
            CommandParseError::InvalidOperandValue
        );
    }

    /// Backslash-escaped quote is preserved as the literal character.
    #[test]
    fn command_parser_unquote_escaped_quote() {
        // Arrange
        let schema = CommandSchemaBuilder::new("cmd").build();

        // Act
        let output = parse(schema, &["it\\'s"]).expect("should parse");

        // Assert
        assert_eq!(output[0].operands, vec![String::from("it's")]);
    }
}
