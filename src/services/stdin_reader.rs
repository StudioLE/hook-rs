//! Reader for the hook input on stdin.

use crate::prelude::*;
use std::io;
use std::io::Read;

/// [`ReadStdin`] from the process stdin.
///
/// - Reads on demand so a read failure becomes an error [`Outcome`] rather than a resolve panic
#[derive(Default, FromServices)]
pub struct StdinReader;

impl ReadStdin for StdinReader {
    fn read_stdin(&self) -> Result<String, Report<ReadStdinError>> {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .change_context(ReadStdinError::Read)?;
        Ok(input)
    }
}
