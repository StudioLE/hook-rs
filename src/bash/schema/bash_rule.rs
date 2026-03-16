//! Rule matching individual Bash commands by name and arguments.

use crate::prelude::*;

/// Rule that matches a [`SimpleContext`] by command name, arguments, and conditions.
#[derive(Default)]
pub struct BashRule {
    /// Unique identifier for this rule.
    pub id: String,
    /// Command name and optional leading arguments to match exactly.
    ///
    /// Examples:
    /// - `head`
    /// - `tail`
    /// - `git status`
    pub command: String,
    /// Only match if any of these arguments are present after the command.
    ///
    /// Examples:
    /// - `Arg::new("-f")`
    /// - `Arg::new("--force")`
    /// - `Arg::new("-X").value("{POST,PUT}")`
    pub with_any: Option<Vec<Arg>>,
    /// Only match if **all** of these arguments are present after the command.
    ///
    /// Examples:
    /// - `[Arg::new("reset"), Arg::new("--hard")]`
    pub with_all: Option<Vec<Arg>>,
    /// Do not match if any of these arguments are present after the command.
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

impl BashRule {
    /// Create a new [`BashRule`] matching the given command.
    pub fn new(id: impl Into<String>, command: impl Into<String>, outcome: Outcome) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            outcome,
            ..Default::default()
        }
    }

    /// Check if this rule matches the given command.
    ///
    /// Single-char short flags (e.g. `-d`) also match inside bundled args (e.g. `-fd`).
    pub fn matches(
        &self,
        simple: &SimpleContext,
        complete: &CompleteContext,
        settings: &Settings,
    ) -> bool {
        let mut parts = self.command.split_whitespace();
        let Some(name) = parts.next() else {
            return false;
        };
        if simple.name != name {
            return false;
        }
        let leading_args: Vec<&str> = parts.collect();
        if !simple
            .args
            .iter()
            .zip(&leading_args)
            .all(|(actual, expected)| actual == expected)
            || simple.args.len() < leading_args.len()
        {
            return false;
        }
        let remaining_args: Vec<&str> = simple
            .args
            .get(leading_args.len()..)
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
            && !condition(simple, complete, settings)
        {
            return false;
        }
        debug!(id = %self.id, decision = %self.outcome.decision, command = %simple.name, "Matched bash rule");
        true
    }
}
