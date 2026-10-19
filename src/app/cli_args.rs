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
