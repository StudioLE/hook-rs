//! Flag definitions for [`ArgParser`].

/// Flag definitions that describe a command's argument structure.
///
/// The schema is position-agnostic: it does not encode where flags
/// may appear relative to subcommands or positional args. The real
/// tool enforces positional constraints; the parser only classifies
/// args for security rule evaluation.
#[derive(Clone, Debug, Default)]
pub struct ArgSchema {
    /// Flags that take no value.
    pub bool_flags: Vec<String>,
    /// Flags that consume the next argument as their value.
    pub value_flags: Vec<String>,
}

impl ArgSchema {
    /// True if the flag takes no value.
    pub fn is_bool_flag(&self, flag: &str) -> bool {
        self.bool_flags.iter().any(|f| f == flag)
    }

    /// True if the flag consumes the next argument as its value.
    pub fn is_value_flag(&self, flag: &str) -> bool {
        self.value_flags.iter().any(|f| f == flag)
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            bool_flags: vec![String::from("-v"), String::from("--verbose")],
            value_flags: vec![
                String::from("-X"),
                String::from("-n"),
                String::from("-o"),
                String::from("--format"),
            ],
        }
    }
}
