//! Schema-aware argument parser.

use crate::prelude::*;

/// Schema-aware argument parser.
///
/// Walks a list of command-line arguments and classifies each one
/// against the provided [`ArgSchema`]. Flags not present in the
/// schema produce an error.
pub struct ArgParser {
    settings: ArgParserSettings,
    queue: VecDeque<String>,
    output: Vec<Arg>,
    after_separator: bool,
}

impl ArgParser {
    /// Create a new [`ArgParser`] with the given settings.
    pub fn new(settings: ArgParserSettings) -> Self {
        Self {
            settings,
            queue: VecDeque::new(),
            output: Vec::new(),
            after_separator: false,
        }
    }

    /// Parse arguments against the schema.
    pub fn parse(
        mut self,
        values: impl IntoIterator<Item = String>,
    ) -> Result<Vec<Arg>, Report<ArgParseError>> {
        self.init(values);
        while let Some(raw) = self.next() {
            if self.after_separator {
                self.push_operand(raw);
                continue;
            }
            if raw == "--" {
                self.push_separator();
                self.after_separator = true;
                continue;
            }
            if let Some(flag) = raw.strip_prefix("--")
                && !flag.is_empty()
            {
                self.parse_long_flag(raw)?;
                continue;
            }
            if raw.starts_with('-') && raw.len() >= 2 {
                self.parse_short_flags(raw)?;
                continue;
            }
            self.push_operand(raw);
        }
        if self.settings.unquote {
            self.unquote_values();
        }
        Ok(self.output)
    }

    fn init<T>(&mut self, values: T)
    where
        T: IntoIterator,
        T::Item: Into<String>,
    {
        self.queue = values.into_iter().map(Into::into).collect();
    }

    fn next(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    /// Parse a single long flag and optional value.
    ///
    /// - Splits on `=` first to handle `--flag=value` inline form
    /// - Falls back to consuming the next argument for value flags
    /// - Accepts bare flag for boolean flags
    /// - Rejects anything not in the schema
    fn parse_long_flag(&mut self, raw: String) -> Result<(), Report<ArgParseError>> {
        debug_assert!(raw.starts_with("--"));
        if let Some((flag, value)) = raw.split_once('=') {
            if self.settings.schema.is_value_flag(flag) {
                self.push_pair(flag, value);
                return Ok(());
            }
            return Err(flag_error(ArgParseError::UnknownFlag, raw));
        }
        if self.settings.schema.is_value_flag(&raw) {
            let value = self
                .next()
                .ok_or_else(|| flag_error(ArgParseError::MissingValue, &raw))?;
            self.push_pair(raw, value);
            return Ok(());
        }
        if self.settings.schema.is_bool_flag(&raw) {
            self.push_flag(raw);
            return Ok(());
        }
        Err(flag_error(ArgParseError::UnknownFlag, raw))
    }

    /// Parse short flags, expanding bundled forms like `-abc` or `-an5`.
    ///
    /// - Iterates characters after the leading `-`
    /// - Value flags consume the remaining characters as an inline value,
    ///   or the next argument if at the end of the bundle
    /// - Bool flags are emitted individually and iteration continues
    fn parse_short_flags(&mut self, raw: String) -> Result<(), Report<ArgParseError>> {
        debug_assert!(raw.starts_with('-'));
        let chars: Vec<char> = raw.chars().skip(1).collect();
        for (i, &c) in chars.iter().enumerate() {
            let flag = format!("-{c}");
            if self.settings.schema.is_value_flag(&flag) {
                let remainder: String = chars
                    .get(i + 1..)
                    .expect("i + 1 is within bounds of enumerated slice")
                    .iter()
                    .collect();
                let value = if remainder.is_empty() {
                    self.next()
                        .ok_or_else(|| flag_error(ArgParseError::MissingValue, &flag))?
                } else {
                    remainder
                };
                self.push_pair(flag, value);
                return Ok(());
            }
            if self.settings.schema.is_bool_flag(&flag) {
                self.push_flag(flag);
                continue;
            }
            return Err(flag_error(ArgParseError::UnknownFlag, &flag));
        }
        Ok(())
    }

    /// Append a boolean flag.
    fn push_flag(&mut self, flag: impl Into<String>) {
        let flag = flag.into();
        debug_assert!(flag.starts_with('-'));
        self.output.push(Arg::Flag(flag));
    }

    /// Append a flag with its value.
    fn push_pair(&mut self, flag: impl Into<String>, value: impl Into<String>) {
        let flag = flag.into();
        debug_assert!(flag.starts_with('-'));
        self.output.push(Arg::FlagPair(flag, value.into()));
    }

    /// Append a positional argument.
    fn push_operand(&mut self, value: impl Into<String>) {
        self.output.push(Arg::Operand(value.into()));
    }

    /// Append the end-of-options separator.
    fn push_separator(&mut self) {
        self.output.push(Arg::Separator);
    }

    /// Apply unquoting to all values.
    fn unquote_values(&mut self) {
        for arg in &mut self.output {
            match arg {
                Arg::FlagPair(_, value) | Arg::Operand(value) => {
                    *value = unquote_str(value);
                }
                _ => {}
            }
        }
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        ArgParser::new(ArgParserSettings::mock())
    }
}

fn flag_error(error: ArgParseError, flag: impl Into<String>) -> Report<ArgParseError> {
    Report::new(error).attach("flag", flag)
}

/// Errors returned by [`ArgParser::parse`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ArgParseError {
    /// Flag not present in the schema.
    #[error("Unknown flag")]
    UnknownFlag,
    /// Value flag at end of args with no following value.
    #[error("Missing value for flag")]
    MissingValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_parser_parse_positional_only() {
        let input = args(&["log", "main"]);
        let args = ArgParser::mock().parse(input).expect("should parse");
        assert_eq!(
            args,
            vec![
                Arg::Operand("log".to_owned()),
                Arg::Operand("main".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_empty() {
        let output = ArgParser::mock().parse(args(&[])).expect("should parse");
        assert_eq!(output, vec![]);
    }

    #[test]
    fn arg_parser_parse_bool_flag_short() {
        let output = ArgParser::mock()
            .parse(args(&["-v", "log"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![Arg::Flag("-v".to_owned()), Arg::Operand("log".to_owned()),]
        );
    }

    #[test]
    fn arg_parser_parse_bool_flag_long() {
        let output = ArgParser::mock()
            .parse(args(&["--verbose", "log"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Flag("--verbose".to_owned()),
                Arg::Operand("log".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_value_flag_short() {
        let output = ArgParser::mock()
            .parse(args(&["-n", "5", "log"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::FlagPair("-n".to_owned(), "5".to_owned()),
                Arg::Operand("log".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_value_flag_long_separate() {
        let output = ArgParser::mock()
            .parse(args(&["--format", "%H", "log"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::FlagPair("--format".to_owned(), "%H".to_owned()),
                Arg::Operand("log".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_value_flag_long_equals() {
        let output = ArgParser::mock()
            .parse(args(&["--format=%H", "log"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::FlagPair("--format".to_owned(), "%H".to_owned()),
                Arg::Operand("log".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_separator() {
        let output = ArgParser::mock()
            .parse(args(&["--", "-v", "log"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Separator,
                Arg::Operand("-v".to_owned()),
                Arg::Operand("log".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_separator_after_flags() {
        let output = ArgParser::mock()
            .parse(args(&["-v", "--", "--unknown"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Flag("-v".to_owned()),
                Arg::Separator,
                Arg::Operand("--unknown".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_bundled_bool_flags() {
        let settings = ArgParserSettings {
            schema: ArgSchema {
                bool_flags: vec!["-a".to_owned(), "-b".to_owned(), "-c".to_owned()],
                value_flags: vec![],
            },
            unquote: false,
        };
        let output = ArgParser::new(settings)
            .parse(args(&["-abc"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Flag("-a".to_owned()),
                Arg::Flag("-b".to_owned()),
                Arg::Flag("-c".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_bundled_value_flag_last_with_next_arg() {
        let settings = ArgParserSettings {
            schema: ArgSchema {
                bool_flags: vec!["-a".to_owned()],
                value_flags: vec!["-n".to_owned()],
            },
            unquote: false,
        };
        let output = ArgParser::new(settings)
            .parse(args(&["-an", "5"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Flag("-a".to_owned()),
                Arg::FlagPair("-n".to_owned(), "5".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_bundled_value_flag_with_inline_value() {
        let settings = ArgParserSettings {
            schema: ArgSchema {
                bool_flags: vec!["-a".to_owned()],
                value_flags: vec!["-n".to_owned()],
            },
            unquote: false,
        };
        let output = ArgParser::new(settings)
            .parse(args(&["-an5"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Flag("-a".to_owned()),
                Arg::FlagPair("-n".to_owned(), "5".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_bundled_value_flag_after_bool_flag() {
        let output = ArgParser::mock()
            .parse(args(&["-vXPOST"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![
                Arg::Flag("-v".to_owned()),
                Arg::FlagPair("-X".to_owned(), "POST".to_owned()),
            ]
        );
    }

    #[test]
    fn arg_parser_parse_bundled_value_flag_consumes_remainder() {
        let output = ArgParser::mock()
            .parse(args(&["-XPOSTv"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![Arg::FlagPair("-X".to_owned(), "POSTv".to_owned()),]
        );
    }

    #[test]
    fn arg_parser_parse_unknown_flag_short() {
        let error = ArgParser::mock()
            .parse(args(&["-z"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::UnknownFlag);
    }

    #[test]
    fn arg_parser_parse_unknown_flag_long() {
        let error = ArgParser::mock()
            .parse(args(&["--unknown"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::UnknownFlag);
    }

    #[test]
    fn arg_parser_parse_missing_value_short() {
        let error = ArgParser::mock()
            .parse(args(&["-n"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::MissingValue);
    }

    #[test]
    fn arg_parser_parse_missing_value_long() {
        let error = ArgParser::mock()
            .parse(args(&["--format"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::MissingValue);
    }

    #[test]
    fn arg_parser_parse_unquote_positional() {
        let output = ArgParser::mock()
            .parse(args(&["\"log\""]))
            .expect("should parse");
        assert_eq!(output, vec![Arg::Operand("log".to_owned())]);
    }

    #[test]
    fn arg_parser_parse_unquote_flag_value() {
        let output = ArgParser::mock()
            .parse(args(&["--format", "'%H'"]))
            .expect("should parse");
        assert_eq!(
            output,
            vec![Arg::FlagPair("--format".to_owned(), "%H".to_owned())]
        );
    }

    #[test]
    fn arg_parser_parse_no_unquote() {
        let settings = ArgParserSettings {
            schema: ArgSchema::mock(),
            unquote: false,
        };
        let output = ArgParser::new(settings)
            .parse(args(&["\"log\""]))
            .expect("should parse");
        assert_eq!(output, vec![Arg::Operand("\"log\"".to_owned())]);
    }

    #[test]
    fn arg_parser_parse_bundled_unknown_flag() {
        let settings = ArgParserSettings {
            schema: ArgSchema {
                bool_flags: vec!["-a".to_owned()],
                value_flags: vec![],
            },
            unquote: false,
        };
        let error = ArgParser::new(settings)
            .parse(args(&["-az"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::UnknownFlag);
    }

    #[test]
    fn arg_parser_parse_bundled_value_flag_last_no_next_arg() {
        let settings = ArgParserSettings {
            schema: ArgSchema {
                bool_flags: vec!["-a".to_owned()],
                value_flags: vec!["-n".to_owned()],
            },
            unquote: false,
        };
        let error = ArgParser::new(settings)
            .parse(args(&["-an"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::MissingValue);
    }

    #[test]
    fn arg_parser_parse_bool_flag_long_with_equals() {
        let error = ArgParser::mock()
            .parse(args(&["--verbose=true"]))
            .expect_err("should fail");
        assert_eq!(*error.current_context(), ArgParseError::UnknownFlag);
    }

    fn args(input: &[&str]) -> Vec<String> {
        input.iter().map(|s| (*s).to_owned()).collect()
    }
}
