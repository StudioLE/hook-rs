//! Rule evaluation: matches parsed commands against registered rules.

use crate::prelude::*;

/// Rule engine that evaluates parsed shell commands against registered security rules.
#[derive(FromServices)]
pub struct BashEvaluator {
    /// User settings for path classification and trusted directories.
    settings: Arc<Settings>,
    /// Registered security rules for matching commands.
    rules: Arc<BashRuleProvider>,
    /// Factory for building path-matching rules.
    paths: Arc<PathRuleFactory>,
}

impl BashEvaluator {
    /// Parse and evaluate a shell command string against all registered rules.
    pub fn evaluate_str(&self, command: &str) -> Result<Outcome, Report<ParseError>> {
        let context = BashParser::new().parse(command)?;
        self.evaluate_all(&context)
    }

    fn evaluate_all(&self, context: &CompleteContext) -> Result<Outcome, Report<ParseError>> {
        let outcomes = self.evaluate_rules(context)?;
        apply_precedence(outcomes)
    }

    fn evaluate_rules(
        &self,
        complete_context: &CompleteContext,
    ) -> Result<Vec<Outcome>, Report<ParseError>> {
        if let Some(outcome) = semicolon_rule(complete_context) {
            return Ok(vec![outcome]);
        }
        let mut all_outcomes = Vec::new();
        let mut has_unmatched = false;
        for simple_context in complete_context.all_commands() {
            let mut outcomes = Vec::new();
            let ctx = BashRuleContext {
                simple: simple_context,
                complete: complete_context,
                settings: &self.settings,
                paths: &self.paths,
            };
            for rule in self.rules.get() {
                if rule.matches(&ctx) {
                    outcomes.push(rule.outcome.clone());
                }
            }
            if outcomes.is_empty() {
                has_unmatched = true;
            }
            all_outcomes.extend(outcomes);
        }
        if has_unmatched
            && !all_outcomes.is_empty()
            && all_outcomes.iter().all(|o| o.decision == Decision::Allow)
        {
            return Err(ParseError::skip(SkipReason::OnlyAllowAll));
        }
        Ok(all_outcomes)
    }
}

/// Merge an outcome into the accumulated result using Deny > Ask > Allow precedence.
fn apply_precedence(mut outcomes: Vec<Outcome>) -> Result<Outcome, Report<ParseError>> {
    if outcomes.is_empty() {
        return Err(ParseError::skip(SkipReason::NoMatches));
    }
    if outcomes.len() == 1 {
        return Ok(outcomes.pop().expect("should be 1 outcome"));
    }
    let outcomes = sort_outcomes(outcomes);
    debug!(
        deny = outcomes.get(&Decision::Deny).unwrap_or(&Vec::new()).len(),
        ask = outcomes.get(&Decision::Ask).unwrap_or(&Vec::new()).len(),
        allow = outcomes.get(&Decision::Allow).unwrap_or(&Vec::new()).len(),
        "Applying precedence"
    );
    if let Some(reasons) = outcomes.get(&Decision::Deny) {
        return Ok(Outcome::combined(Decision::Deny, reasons));
    }
    if let Some(reasons) = outcomes.get(&Decision::Ask) {
        return Ok(Outcome::combined(Decision::Ask, reasons));
    }
    if let Some(reasons) = outcomes.get(&Decision::Allow) {
        return Ok(Outcome::combined(Decision::Allow, reasons));
    }
    unreachable!("should be at least one decision");
}

/// Group outcomes by [`Decision`] variant.
fn sort_outcomes(outcomes: Vec<Outcome>) -> HashMap<Decision, Vec<String>> {
    let mut map = HashMap::new();
    for outcome in outcomes {
        let entry = map.entry(outcome.decision).or_insert_with(Vec::new);
        entry.push(outcome.reason);
    }
    map
}

#[cfg(test)]
impl BashEvaluator {
    /// Create an evaluator with the given settings for testing.
    pub(crate) fn mock() -> Arc<Self> {
        ServiceProvider::mock().expect()
    }
}

/// Parse and evaluate `command` with only the given rules.
#[cfg(test)]
pub(crate) fn eval_rules(
    rules: Vec<BashRule>,
    command: &str,
) -> Result<Outcome, Report<ParseError>> {
    ServiceBuilder::mock()
        .with_instance(BashRuleProvider::new(rules))
        .build()
        .expect::<BashEvaluator>()
        .evaluate_str(command)
}

/// Parse and evaluate `command` with the given rules and custom [`Settings`].
#[cfg(test)]
pub(crate) fn eval_rules_with_settings(
    rules: Vec<BashRule>,
    command: &str,
    settings: Settings,
) -> Result<Outcome, Report<ParseError>> {
    ServiceBuilder::mock()
        .with_instance(settings)
        .with_instance(BashRuleProvider::new(rules))
        .build()
        .expect::<BashEvaluator>()
        .evaluate_str(command)
}

/// Extract a [`SkipReason`] from [`ParseError::Skip`] or panic.
#[cfg(test)]
#[expect(clippy::panic, reason = "test helper")]
pub(crate) fn expect_skip(result: Result<Outcome, Report<ParseError>>) -> SkipReason {
    match result
        .expect_err("command should not succeed")
        .current_context()
    {
        ParseError::Skip(reason) => *reason,
        other => panic!("expected Skip, got {other:?}"),
    }
}

/// Expect an [`Outcome`] or panic.
#[cfg(test)]
pub(crate) fn expect_outcome(result: Result<Outcome, Report<ParseError>>) -> Outcome {
    result.expect("command should succeed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_git() {
        let result = BashEvaluator::mock().evaluate_str("git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rm_rf_tmp() {
        let result = BashEvaluator::mock().evaluate_str("rm -rf /tmp/nothing");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn stash_pop() {
        let result = BashEvaluator::mock().evaluate_str("git stash pop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn reset_hard() {
        let result = BashEvaluator::mock().evaluate_str("git reset --hard");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn checkout_discard() {
        let result = BashEvaluator::mock().evaluate_str("git checkout -- file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_and_push() {
        let result = BashEvaluator::mock().evaluate_str("git commit -m 'msg' && git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn echo_separator() {
        let result = BashEvaluator::mock().evaluate_str("cmd && echo \"---\"");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn find_delete() {
        let result = BashEvaluator::mock().evaluate_str("find . -name '*.tmp' -delete");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn insta_heredoc() {
        let result = BashEvaluator::mock().evaluate_str("cargo insta review <<EOF\na\nEOF");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_and_git() {
        let result = BashEvaluator::mock().evaluate_str("cd /path && git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn plain_ls() {
        let result = BashEvaluator::mock().evaluate_str("ls -la");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn standalone_push() {
        let result = BashEvaluator::mock().evaluate_str("git push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_read() {
        let result = BashEvaluator::mock().evaluate_str("git branch -a");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_write() {
        let result = BashEvaluator::mock().evaluate_str("git branch -d old");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_read() {
        let result = BashEvaluator::mock().evaluate_str("git tag -l");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_create() {
        let result = BashEvaluator::mock().evaluate_str("git tag v1.0");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_verbose() {
        let result = BashEvaluator::mock().evaluate_str("git remote -v");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add() {
        let result = BashEvaluator::mock().evaluate_str("git remote add upstream https://x.com");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rm_tmp_file() {
        let result = BashEvaluator::mock().evaluate_str("rm /tmp/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d() {
        let result = BashEvaluator::mock().evaluate_str("git clean -fd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn forked_path() {
        let result =
            BashEvaluator::mock().evaluate_str("git -C /home/user/repos/forked/repo status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_path() {
        let result = BashEvaluator::mock().evaluate_str("git -C /tmp/sketchy status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_pop() {
        let result =
            BashEvaluator::mock().evaluate_str("git -C /home/user/repos/my-project stash pop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_reset_hard() {
        let result =
            BashEvaluator::mock().evaluate_str("git -C /home/user/repos/my-project reset --hard");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_checkout_discard() {
        let result = BashEvaluator::mock()
            .evaluate_str("git -C /home/user/repos/my-project checkout -- file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_git_clean_d() {
        let result =
            BashEvaluator::mock().evaluate_str("git -C /home/user/repos/my-project clean -fd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_status_piped() {
        let result = BashEvaluator::mock().evaluate_str("git status | head -5");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_diff_and_status() {
        let result = BashEvaluator::mock().evaluate_str("git diff && git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn safe_and_unknown() {
        let result = BashEvaluator::mock().evaluate_str("git status && cargo publish");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn semi_both_safe() {
        let result = BashEvaluator::mock().evaluate_str("git status ; git diff");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_safe_and_unknown() {
        let result = BashEvaluator::mock().evaluate_str("git status ; cargo publish");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_safe_and_rm() {
        let result = BashEvaluator::mock().evaluate_str("git status ; rm -rf /tmp/nothing");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_mixed_with_and() {
        let result = BashEvaluator::mock().evaluate_str("git status && git diff ; git log");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn for_loop_echo() {
        let result = BashEvaluator::mock().evaluate_str("for f in *.txt; do echo $f; done");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_safe_git() {
        let result = BashEvaluator::mock().evaluate_str("for f in *.txt; do git status; done");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_rm() {
        let result = BashEvaluator::mock().evaluate_str("for f in *.tmp; do rm $f; done");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn for_loop_safe_and_unknown() {
        let result = BashEvaluator::mock()
            .evaluate_str("for f in *.txt; do git status && cargo publish; done");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn for_loop_unknown() {
        let result = BashEvaluator::mock().evaluate_str("for f in *.txt; do cargo publish; done");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
