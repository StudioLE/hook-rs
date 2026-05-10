//! Subcommand dispatch via resolved handlers.

use crate::prelude::*;

/// Dispatch the selected subcommand to its handler.
#[derive(FromServices)]
pub struct SubcommandHandler {
    cli_options: Arc<CliOptions>,
    bash: Arc<BashHandler>,
    glob: Arc<GlobHandler>,
    grep: Arc<GrepHandler>,
    read: Arc<ReadHandler>,
}

impl SubcommandHandler {
    /// Dispatch to the handler matching the selected subcommand.
    pub fn run(&self) -> Option<Outcome> {
        match &self.cli_options.subcommand {
            Subcommand::Bash(_) => run_handler(self.bash.as_ref()),
            Subcommand::Glob(_) => run_handler(self.glob.as_ref()),
            Subcommand::Grep(_) => run_handler(self.grep.as_ref()),
            Subcommand::Read(_) => run_handler(self.read.as_ref()),
        }
    }
}

/// Run the handler.
///
/// - Deserialize stdin as [`HookInput`]
/// - Returns the handler's outcome on success
/// - Logs and converts deserialization failures to an error [`Outcome`]
fn run_handler<H: Handler>(handler: &H) -> Option<Outcome> {
    match HookInput::<H::Input>::from_stdin() {
        Ok(input) => handler.run(input.tool_input),
        Err(report) => {
            error!("{}", report.render());
            Some(Outcome::error(report))
        }
    }
}
