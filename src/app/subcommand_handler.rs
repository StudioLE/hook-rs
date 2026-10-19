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
    ///
    /// - Logs and converts deserialization failures to an error [`Outcome`]
    pub fn run(&self) -> Option<Outcome> {
        self.try_run().unwrap_or_else(|report| {
            error!("{}", report.render());
            Some(Outcome::error(report))
        })
    }

    /// Run the handler matching the selected subcommand with the hook input.
    fn try_run(&self) -> Result<Option<Outcome>, Report<HookError>> {
        let outcome = match self.cli_args.subcommand {
            Subcommand::Bash => self.bash.run(read_input()?),
            Subcommand::Glob => self.glob.run(read_input()?),
            Subcommand::Grep => self.grep.run(read_input()?),
            Subcommand::Read => self.read.run(read_input()?),
        };
        Ok(outcome)
    }
}

/// Read and deserialize the hook input from stdin.
///
/// - Logs the session working directory and permission mode
fn read_input<T: DeserializeOwned>() -> Result<HookInput<T>, Report<HookError>> {
    let input = HookInput::from_stdin()?;
    debug!(cwd = ?input.cwd, permission_mode = ?input.permission_mode, "Received hook input");
    Ok(input)
}
