//! Prefix-based rule matching individual simple commands.

use crate::prelude::*;

/// Rule that matches a [`SimpleContext`] by command prefix, options, and conditions.
#[derive(Default)]
pub struct SimpleRule {
    /// Unique identifier for this rule.
    #[expect(dead_code, reason = "id for planned matching")]
    pub id: String,
    /// Match commands that start with prefix.
    ///
    /// Examples:
    /// - `head`
    /// - `tail`
    /// - `git status`
    pub prefix: String,
    /// Only match if any of these arguments are present after the prefix.
    ///
    /// Examples:
    /// - `Arg::new("-f")`
    /// - `Arg::new("--force")`
    /// - `Arg::new("-X").value("{POST,PUT}")`
    pub with_any: Option<Vec<Arg>>,
    /// Only match if **all** of these arguments are present after the prefix.
    ///
    /// Examples:
    /// - `[Arg::new("reset"), Arg::new("--hard")]`
    pub with_all: Option<Vec<Arg>>,
    /// Do not match if any of these arguments are present after the prefix.
    ///
    /// Examples:
    /// - `Arg::new("-i")`
    /// - `Arg::new("--in-place")`
    pub without_any: Option<Vec<Arg>>,
    /// Only match if the command satisfies this condition.
    pub condition: Option<fn(&SimpleContext, &CompleteContext, &Settings) -> bool>,
    /// Outcome if the command matches.
    pub outcome: Outcome,
}

impl SimpleRule {
    /// Create a new [`SimpleRule`] matching the given prefix.
    pub fn new(id: impl Into<String>, prefix: impl Into<String>, outcome: Outcome) -> Self {
        Self {
            id: id.into(),
            prefix: prefix.into(),
            outcome,
            ..Default::default()
        }
    }

    /// Check if this rule matches the given command.
    ///
    /// Single-char short flags (e.g. `-d`) also match inside bundled args (e.g. `-fd`).
    pub fn matches(
        &self,
        cmd: &SimpleContext,
        complete: &CompleteContext,
        settings: &Settings,
    ) -> bool {
        let mut parts = self.prefix.split_whitespace();
        let Some(name) = parts.next() else {
            return false;
        };
        if cmd.name != name {
            return false;
        }
        let prefix_args: Vec<&str> = parts.collect();
        if !cmd
            .args
            .iter()
            .zip(&prefix_args)
            .all(|(actual, expected)| actual == expected)
            || cmd.args.len() < prefix_args.len()
        {
            return false;
        }
        let remaining_args: Vec<&str> = cmd
            .args
            .get(prefix_args.len()..)
            .unwrap_or_default()
            .iter()
            .map(String::as_str)
            .collect();
        if let Some(with) = &self.with_any
            && !with.iter().any(|a| a.is_present(&remaining_args))
        {
            return false;
        }
        if let Some(all) = &self.with_all
            && !all.iter().all(|a| a.is_present(&remaining_args))
        {
            return false;
        }
        if let Some(without) = &self.without_any
            && without.iter().any(|a| a.is_present(&remaining_args))
        {
            return false;
        }
        if let Some(condition) = &self.condition
            && !condition(cmd, complete, settings)
        {
            return false;
        }
        true
    }
}
