//! CLI subcommand types for the hook binary.

use crate::prelude::*;
use argh::FromArgs;
use tracing::Level;

/// Claude Code hook evaluator.
#[derive(FromArgs)]
pub struct Cli {
    #[argh(subcommand)]
    subcommand: Subcommand,

    /// log level
    #[argh(option)]
    log_level: Option<Level>,
}

/// Tool-specific subcommand.
#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Bash(BashCmd),
    Glob(GlobCmd),
    Grep(GrepCmd),
    Read(ReadCmd),
}

/// Evaluate a Bash tool call
#[derive(FromArgs)]
#[argh(subcommand, name = "bash")]
struct BashCmd {}

/// Evaluate a Glob tool call
#[derive(FromArgs)]
#[argh(subcommand, name = "glob")]
struct GlobCmd {}

/// Evaluate a Grep tool call
#[derive(FromArgs)]
#[argh(subcommand, name = "grep")]
struct GrepCmd {}

/// Evaluate a Read tool call
#[derive(FromArgs)]
#[argh(subcommand, name = "read")]
struct ReadCmd {}

impl Cli {
    /// Main entrypoint for the hook binary.
    ///
    /// - Parse CLI arguments
    /// - Dispatch to the appropriate handler
    /// - Print the result
    pub fn run() {
        let cli: Cli = argh::from_env();
        let _logger = init_logger(cli.log_level);
        let outcome = match cli.subcommand {
            Subcommand::Bash(_) => run::<BashHandler>(),
            Subcommand::Glob(_) => run::<GlobHandler>(),
            Subcommand::Grep(_) => run::<GrepHandler>(),
            Subcommand::Read(_) => run::<ReadHandler>(),
        };
        if let Some(outcome) = outcome {
            info!("{outcome}");
            outcome.print_hook_output();
        } else {
            info!("No outcome");
        }
    }
}
