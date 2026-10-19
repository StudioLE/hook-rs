//! Subcommand dispatch via resolved handlers.

use crate::prelude::*;

/// Dispatch the selected subcommand to its handler.
#[derive(FromServices)]
pub struct SubcommandHandler {
    cli_args: Arc<CliArgs>,
    stdin: Arc<StdinReader>,
    bash: Arc<BashHandler>,
    glob: Arc<GlobHandler>,
    grep: Arc<GrepHandler>,
    read: Arc<ReadHandler>,
}

impl SubcommandHandler {
    /// Dispatch to the handler matching the selected subcommand.
    ///
    /// - Logs and converts read or deserialization failures to an error [`Outcome`]
    pub fn run(&self) -> Option<Outcome> {
        self.try_run().unwrap_or_else(|report| {
            error!("{}", report.render());
            Some(Outcome::error(report))
        })
    }

    /// Run the handler matching the selected subcommand with the hook input.
    fn try_run(&self) -> Result<Option<Outcome>, Report<HookError>> {
        let outcome = match self.cli_args.subcommand {
            Subcommand::Bash => self.bash.run(self.read_input()?),
            Subcommand::Glob => self.glob.run(self.read_input()?),
            Subcommand::Grep => self.grep.run(self.read_input()?),
            Subcommand::Read => self.read.run(self.read_input()?),
        };
        Ok(outcome)
    }

    /// Read and deserialize the hook input.
    ///
    /// - Logs the session working directory and permission mode
    fn read_input<T: DeserializeOwned>(&self) -> Result<HookInput<T>, Report<HookError>> {
        let json = self.stdin.read()?;
        let input = HookInput::from_json(&json)?;
        debug!(cwd = ?input.cwd, permission_mode = ?input.permission_mode, "Received hook input");
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A matching rule returns its decision.
    #[test]
    fn subcommand_handler_run() {
        let handler = create_handler(r#"{"tool_input":{"command":"rm -rf /"}}"#);
        let outcome = handler.run();
        assert_eq!(outcome.expect("should match").decision, Decision::Deny);
    }

    /// Hook input that is not valid JSON falls back to asking the user.
    #[test]
    fn subcommand_handler_run_invalid_input() {
        let handler = create_handler("not json");
        let outcome = handler.run();
        assert_eq!(outcome.expect("should match").decision, Decision::Ask);
    }

    /// Resolve a [`SubcommandHandler`] reading `input` as stdin.
    fn create_handler(input: &str) -> Arc<SubcommandHandler> {
        ServiceBuilder::mock()
            .with_instance(StdinReader::new(input))
            .build()
            .expect_init()
            .expect::<SubcommandHandler>()
    }
}
