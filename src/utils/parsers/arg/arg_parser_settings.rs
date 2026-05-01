//! Settings for [`ArgParser`].

use crate::prelude::*;

/// Settings for [`ArgParser`].
pub struct ArgParserSettings {
    /// Flag definitions for this command.
    pub schema: ArgSchema,
    /// Apply [`unquote_str`] to values before storing them.
    pub unquote: bool,
}

impl ArgParserSettings {
    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            schema: ArgSchema::mock(),
            unquote: true,
        }
    }
}
