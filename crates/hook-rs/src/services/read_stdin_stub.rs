//! Stub for reading the raw hook input.

use crate::prelude::*;

/// [`ReadStdin`] returning a fixed input or [`ReadStdinError`].
///
/// - Defaults to a Bash `git status` hook input to match [`CliArgs::fixture`]
pub struct ReadStdinStub {
    /// Value returned by [`ReadStdin::read_stdin`].
    pub input: Result<String, ReadStdinError>,
}

impl FromServices for ReadStdinStub {
    type Error = Infallible;

    fn from_services(_services: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self {
            input: Ok(String::from(r#"{"tool_input":{"command":"git status"}}"#)),
        })
    }
}

impl ReadStdin for ReadStdinStub {
    fn read_stdin(&self) -> Result<String, Report<ReadStdinError>> {
        self.input.clone().map_err(Report::new)
    }
}
