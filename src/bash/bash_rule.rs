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
    /// - `ArgMatcher::new("-f")`
    /// - `ArgMatcher::new("--force")`
    /// - `ArgMatcher::new("-X").value("{POST,PUT}")`
    pub with_any: Option<Vec<ArgMatcher>>,
    /// Only match if **all** of these arguments are present after the command.
    ///
    /// Examples:
    /// - `[ArgMatcher::new("reset"), ArgMatcher::new("--hard")]`
    pub with_all: Option<Vec<ArgMatcher>>,
    /// Do not match if any of these arguments are present after the command.
    ///
    /// Examples:
    /// - `ArgMatcher::new("-i")`
    /// - `ArgMatcher::new("--in-place")`
    pub without_any: Option<Vec<ArgMatcher>>,
    /// Only match if the command satisfies this condition.
    pub condition: Option<fn(&BashRuleContext) -> bool>,
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
    pub fn matches(&self, ctx: &BashRuleContext) -> bool {
        let Some(leading_count) = self.get_leading_count(ctx) else {
            return false;
        };
        self.matches_args(ctx, leading_count)
    }

    /// Get the outcome of this rule for the given command.
    ///
    /// - Returns the [`DenyReason::VariableArg`] deny if the command has a variable
    ///   and this rule checks arguments, since the variable's value is unknown
    /// - Returns [`BashRule::outcome`] if the rule matches
    pub fn get_outcome(&self, ctx: &BashRuleContext) -> Option<Outcome> {
        let leading_count = self.get_leading_count(ctx)?;
        if ctx.simple.has_variable && self.has_arg_checks() {
            return Some(Outcome::deny(DenyReason::VariableArg.to_string()));
        }
        self.matches_args(ctx, leading_count)
            .then(|| self.outcome.clone())
    }

    fn matches_args(&self, ctx: &BashRuleContext, leading_count: usize) -> bool {
        let args: Vec<&str> = ctx
            .simple
            .args
            .get(leading_count..)
            .expect("leading count should not exceed args")
            .iter()
            .map(String::as_str)
            .collect();
        if !(self.matches_with_any(&args)
            && self.matches_with_all(&args)
            && self.matches_without_any(&args)
            && self.matches_condition(ctx))
        {
            return false;
        }
        debug!(id = %self.id, decision = %self.outcome.decision, command = %ctx.simple.name, "Matched bash rule");
        true
    }

    /// Count of leading args matching [`BashRule::command`].
    ///
    /// - `None` if the command name or leading args don't match
    fn get_leading_count(&self, ctx: &BashRuleContext) -> Option<usize> {
        let mut words = self.command.split_whitespace();
        if words.next()? != ctx.simple.name {
            return None;
        }
        let leading: Vec<&str> = words.collect();
        let actual = ctx.simple.args.get(..leading.len())?;
        (actual == leading.as_slice()).then_some(leading.len())
    }

    /// Does this rule check arguments with `with_any`, `with_all`, or `without_any`?
    fn has_arg_checks(&self) -> bool {
        self.with_any.is_some() || self.with_all.is_some() || self.without_any.is_some()
    }

    /// Is any `with_any` arg present?
    fn matches_with_any(&self, args: &[&str]) -> bool {
        self.with_any
            .as_ref()
            .is_none_or(|with| with.iter().any(|a| a.is_present(args)))
    }

    /// Are all `with_all` args present?
    fn matches_with_all(&self, args: &[&str]) -> bool {
        self.with_all
            .as_ref()
            .is_none_or(|all| all.iter().all(|a| a.is_present(args)))
    }

    /// Are all `without_any` args absent?
    fn matches_without_any(&self, args: &[&str]) -> bool {
        self.without_any
            .as_ref()
            .is_none_or(|without| !without.iter().any(|a| a.is_present(args)))
    }

    /// Does the command satisfy `condition`?
    fn matches_condition(&self, ctx: &BashRuleContext) -> bool {
        self.condition.is_none_or(|condition| condition(ctx))
    }
}
