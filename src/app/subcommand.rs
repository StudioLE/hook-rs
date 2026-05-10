//! Subcommand variants for the hook CLI.
//!
//! Empty per-variant structs are required by `argh` which does not support unit enum variants.

use argh::FromArgs;

/// Tool-specific subcommand.
#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Subcommand {
    Bash(BashArgs),
    Glob(GlobArgs),
    Grep(GrepArgs),
    Read(ReadArgs),
}

/// Evaluate a Bash tool call.
#[derive(FromArgs)]
#[argh(subcommand, name = "bash")]
pub struct BashArgs {}

/// Evaluate a Glob tool call.
#[derive(FromArgs)]
#[argh(subcommand, name = "glob")]
pub struct GlobArgs {}

/// Evaluate a Grep tool call.
#[derive(FromArgs)]
#[argh(subcommand, name = "grep")]
pub struct GrepArgs {}

/// Evaluate a Read tool call.
#[derive(FromArgs)]
#[argh(subcommand, name = "read")]
pub struct ReadArgs {}
