//! Hook input source registered in the service container.

use crate::prelude::*;
use std::io;
use std::io::Read;

/// Source of the raw hook input JSON.
///
/// - Reads stdin on demand so a read failure becomes an error [`Outcome`] rather than a resolve panic
pub struct StdinReader {
    /// Fixed input returned in place of stdin.
    input: Option<String>,
}

impl StdinReader {
    /// Read the raw hook input.
    pub fn read(&self) -> Result<String, Report<HookError>> {
        if let Some(input) = &self.input {
            return Ok(input.clone());
        }
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .change_context(HookError::ReadStdin)?;
        Ok(input)
    }

    /// Create a new [`StdinReader`] returning `input` for testing.
    #[cfg(test)]
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: Some(input.into()),
        }
    }
}

impl FromServices for StdinReader {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self { input: None })
    }
}
