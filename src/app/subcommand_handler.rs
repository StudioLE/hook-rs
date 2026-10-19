//! Subcommand dispatch via resolved handlers.

use crate::prelude::*;

/// Dispatch the selected subcommand to its handler.
#[derive(FromServices)]
pub struct SubcommandHandler {
    cli_args: Arc<CliArgs>,
    bash: Arc<BashHandler>,
    glob: Arc<GlobHandler>,
    grep: Arc<GrepHandler>,
    read: Arc<ReadHandler>,
}

impl SubcommandHandler {
    /// Dispatch to the handler matching the selected subcommand.
    pub fn run(&self) -> Option<Outcome> {
        match &self.cli_args.subcommand {
            Subcommand::Bash => run_handler(self.bash.as_ref()),
            Subcommand::Glob => run_handler(self.glob.as_ref()),
            Subcommand::Grep => run_handler(self.grep.as_ref()),
            Subcommand::Read => run_handler(self.read.as_ref()),
        }
    }
}

/// Run the handler.
///
/// - Deserialize stdin as [`HookInput`]
/// - Logs the session working directory and permission mode
/// - Returns the handler's outcome on success
/// - Logs and converts deserialization failures to an error [`Outcome`]
fn run_handler<H: Handler>(handler: &H) -> Option<Outcome> {
    match HookInput::<H::Input>::from_stdin() {
        Ok(input) => {
            debug!(cwd = ?input.cwd, permission_mode = ?input.permission_mode, "Received hook input");
            handler.run(input)
        }
        Err(report) => {
            error!("{}", report.render());
            Some(Outcome::error(report))
        }
    }
}
