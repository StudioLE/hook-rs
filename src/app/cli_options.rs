//! Parsed CLI arguments registered in the service container.

use crate::prelude::*;
use argh::{FromArgs, from_env};

/// Parsed CLI arguments.
#[derive(FromArgs)]
#[argh(name = "hook-rs", description = "Claude Code hook evaluator")]
pub struct CliOptions {
    /// subcommand selecting which tool handler to invoke
    #[argh(subcommand)]
    pub subcommand: Subcommand,
    /// log level
    #[argh(option)]
    pub log_level: Option<LogLevel>,
}

impl FromServices for CliOptions {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(from_env())
    }
}
