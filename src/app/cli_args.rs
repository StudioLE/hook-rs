//! Parsed CLI arguments registered in the service container.

use crate::prelude::*;
use usage_rs::Cli as UsageCli;

/// Parsed CLI arguments.
#[derive(Debug, UsageCli)]
#[usage(bin = "hook-rs", about, version, unknown_flags = "error")]
pub struct CliArgs {
    /// Set the log level.
    #[usage(long, value_enum)]
    pub log_level: Option<LogLevel>,
    /// Tool handler to invoke.
    #[usage(subcommand)]
    pub subcommand: Subcommand,
}

impl FromServices for CliArgs {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(CliArgs::parse())
    }
}

#[cfg(test)]
impl CliArgs {
    /// Deterministic [`CliArgs`] for testing.
    #[must_use]
    pub fn mock() -> Self {
        Self {
            log_level: Some(LogLevel::Trace),
            subcommand: Subcommand::Bash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    /// An unknown flag is rejected, which `parse` turns into exit code `2`.
    ///
    /// - Claude Code blocks the tool call on exit `2`, so a misconfigured hook fails closed
    #[test]
    fn cli_args_try_parse_from_unknown_flag() {
        // Arrange
        let argv = ["hook-rs", "bash", "--oops"].map(OsStr::new);
        // Act
        let parsed = CliArgs::try_parse_from(&argv);
        // Assert
        assert!(parsed.is_err());
    }
}
