//! Option definition within a [`CommandSchema`].

use crate::prelude::*;

/// Define an option (flag) valid at one command level.
#[derive(Clone, Debug)]
pub struct OptionSchema {
    /// Aliases for this option.
    ///
    /// Example: `["-f", "--file"]`
    pub names: Vec<String>,
    /// Constraint on the value this option accepts.
    ///
    /// - `None` means a bool flag (no value)
    /// - `Some(constraint)` means the option takes a value
    pub value: Option<ValueConstraint>,
    /// Whether this option may appear more than once.
    ///
    /// Default: `false`
    pub repeatable: bool,
}

impl OptionSchema {
    /// True if the given flag name matches any of this option's names.
    pub fn matches(&self, flag: &str) -> bool {
        self.names.iter().any(|n| n == flag)
    }

    /// True if this option takes no value.
    pub fn is_bool(&self) -> bool {
        self.value.is_none()
    }

    /// True if this option takes a value.
    pub fn is_value(&self) -> bool {
        self.value.is_some()
    }
}
