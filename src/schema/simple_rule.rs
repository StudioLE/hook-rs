use crate::prelude::*;

#[derive(Default)]
pub struct SimpleRule {
    /// Match commands that start with prefix.
    ///
    /// Examples:
    /// - `head`
    /// - `tail`
    /// - `git status`
    pub prefix: String,
    /// Only match if any of these options are present after the prefix.
    ///
    /// Examples:
    /// - `-f`
    /// - `--force`
    /// - `--output`
    pub with_option: Option<Vec<String>>,
    /// Do not match if any of these options are present after the prefix.
    ///
    /// Examples:
    /// - `-i`
    /// - `--in-place`
    /// - `--output`
    pub without_option: Option<Vec<String>>,
    /// Only match if the command satisfies this condition.
    pub condition: Option<fn(&SimpleContext) -> bool>,
    /// Outcome if the command matches.
    pub outcome: Outcome,
}

impl SimpleRule {
    pub fn new(prefix: impl Into<String>, outcome: Outcome) -> Self {
        Self {
            prefix: prefix.into(),
            outcome,
            ..Default::default()
        }
    }

    /// Check if this rule matches the given command.
    pub fn matches(&self, cmd: &SimpleContext) -> bool {
        let mut parts = self.prefix.split_whitespace();
        let Some(name) = parts.next() else {
            return false;
        };
        if cmd.name != name {
            return false;
        }
        let prefix_args: Vec<&str> = parts.collect();
        if cmd.args.len() < prefix_args.len() {
            return false;
        }
        for (i, expected) in prefix_args.iter().enumerate() {
            if cmd.args[i] != *expected {
                return false;
            }
        }
        let remaining_args: Vec<&str> = cmd.args[prefix_args.len()..]
            .iter()
            .map(String::as_str)
            .collect();
        if let Some(with) = &self.with_option
            && !with
                .iter()
                .any(|opt| remaining_args.contains(&opt.as_str()))
        {
            return false;
        }
        if let Some(without) = &self.without_option
            && without
                .iter()
                .any(|opt| remaining_args.contains(&opt.as_str()))
        {
            return false;
        }
        if let Some(condition) = &self.condition
            && !condition(cmd)
        {
            return false;
        }
        true
    }
}
