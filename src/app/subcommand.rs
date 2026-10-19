//! Subcommand variants for the hook CLI.

use usage_rs::Subcommands as UsageSubcommands;

/// Tool-specific subcommand.
#[derive(Clone, Debug, Eq, PartialEq, UsageSubcommands)]
pub enum Subcommand {
    /// Evaluate a Bash tool call.
    Bash,
    /// Evaluate a Glob tool call.
    Glob,
    /// Evaluate a Grep tool call.
    Grep,
    /// Evaluate a Read tool call.
    Read,
}
