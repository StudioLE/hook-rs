//! Trait for reading the raw hook input.

use crate::prelude::*;

/// Read the raw hook input JSON.
pub trait ReadStdin: Send + Sync {
    /// Read the whole of stdin.
    fn read_stdin(&self) -> Result<String, Report<ReadStdinError>>;
}

/// Errors returned by [`ReadStdin`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReadStdinError {
    /// Unable to read stdin.
    #[error("Unable to read stdin")]
    Read,
}
