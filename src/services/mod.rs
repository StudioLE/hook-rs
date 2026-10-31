//! Shared application services.

mod host_context;
mod read_stdin;
#[cfg(test)]
mod read_stdin_stub;
mod stdin_reader;

pub use host_context::*;
pub use read_stdin::*;
#[cfg(test)]
pub use read_stdin_stub::*;
pub use stdin_reader::*;
