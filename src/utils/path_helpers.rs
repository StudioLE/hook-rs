//! Path resolution for tool inputs with optional path fields.

use std::env::current_dir;

/// Resolve an optional path against the current working directory.
pub trait UnwrapOrCwd {
    /// Resolve an optional path to an absolute string, falling back to the current
    /// working directory when absent.
    fn unwrap_or_cwd(self) -> String;
}

impl UnwrapOrCwd for Option<String> {
    fn unwrap_or_cwd(self) -> String {
        self.unwrap_or_else(cwd)
    }
}

/// Current working directory as an owned string.
pub fn cwd() -> String {
    current_dir()
        .expect("current working dir should be valid")
        .to_string_lossy()
        .to_string()
}
