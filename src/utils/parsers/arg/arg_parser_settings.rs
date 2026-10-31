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
    /// Create an [`ArgParserSettings`] fixture using [`ArgSchema::fixture`].
    #[cfg(test)]
    pub fn fixture() -> Self {
        Self {
            schema: ArgSchema::fixture(),
            unquote: true,
        }
    }
}
