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
    /// - `-f`
    /// - `--force`
    /// - `--output`
    pub with_any: Option<Vec<String>>,
    /// Only match if **all** of these arguments are present after the prefix.
    ///
    /// Examples:
    /// - `["reset", "--hard"]`
    /// - `["stash", "pop"]`
    pub with_all: Option<Vec<String>>,
    /// Do not match if any of these arguments are present after the prefix.
    ///
    /// Examples:
    /// - `-i`
    /// - `--in-place`
    /// - `--output`
    pub without_any: Option<Vec<String>>,
    /// Only match if the command satisfies this condition.
    pub condition: Option<fn(&SimpleContext) -> bool>,
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
    pub fn matches(&self, cmd: &SimpleContext) -> bool {
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
            && !with.iter().any(|opt| arg_matches(&remaining_args, opt))
        {
            return false;
        }
        if let Some(all) = &self.with_all
            && !all.iter().all(|opt| arg_matches(&remaining_args, opt))
        {
            return false;
        }
        if let Some(without) = &self.without_any
            && without.iter().any(|opt| arg_matches(&remaining_args, opt))
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

/// Check if `arg` is present in `args`.
///
/// - Single-char short options (e.g. `-d`) also match inside bundled args (e.g. `-fd`, `-fxd`).
/// - Long flags and non-flag args require an exact match.
fn arg_matches(args: &[&str], arg: &str) -> bool {
    let is_single_short = arg.len() == 2 && arg.starts_with('-') && arg != "--";
    if is_single_short {
        let Some(&ch) = arg.as_bytes().get(1) else {
            return false;
        };
        args.iter().any(|a| {
            a.starts_with('-') && !a.starts_with("--") && a.bytes().skip(1).any(|b| b == ch)
        })
    } else {
        args.contains(&arg)
    }
}
