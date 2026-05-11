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

#[cfg(test)]
/// Parse and evaluate `command`, expecting a successful [`Outcome`].
pub(crate) fn evaluate_expect_outcome(command: &str) -> Outcome {
    BashEvaluator::mock()
        .evaluate_str(command)
        .expect("command should produce an outcome")
}

#[cfg(test)]
/// Parse and evaluate `command`, expecting a [`SkipReason`].
#[expect(clippy::panic, reason = "test helper")]
pub(crate) fn evaluate_expect_skip(command: &str) -> SkipReason {
    match BashEvaluator::mock()
        .evaluate_str(command)
        .expect_err("command should not succeed")
        .current_context()
    {
        ParseError::Skip(reason) => *reason,
        other => panic!("expected Skip, got {other:?}"),
    }
}

/// Parse and evaluate `command` with custom [`Settings`].
#[cfg(test)]
pub(crate) fn eval(command: &str, settings: Settings) -> Result<Outcome, Report<ParseError>> {
    ServiceBuilder::mock()
        .with_instance(settings)
        .build()
        .expect::<BashEvaluator>()
        .evaluate_str(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_git() {
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rm_rf_tmp() {
        let outcome = evaluate_expect_outcome("rm -rf /tmp/nothing");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn stash_pop() {
        let outcome = evaluate_expect_outcome("git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn reset_hard() {
        let outcome = evaluate_expect_outcome("git reset --hard");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn checkout_discard() {
        let outcome = evaluate_expect_outcome("git checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_and_push() {
        let outcome = evaluate_expect_outcome("git commit -m 'msg' && git push");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn echo_separator() {
        let reason = evaluate_expect_skip("cmd && echo \"---\"");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn find_delete() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn insta_heredoc() {
        let outcome = evaluate_expect_outcome("cargo insta review <<EOF\na\nEOF");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_and_git() {
        let outcome = evaluate_expect_outcome("cd /path && git status");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn plain_ls() {
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn standalone_push() {
        let reason = evaluate_expect_skip("git push");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_read() {
        let outcome = evaluate_expect_outcome("git branch -a");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_write() {
        let reason = evaluate_expect_skip("git branch -d old");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_read() {
        let outcome = evaluate_expect_outcome("git tag -l");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_create() {
        let reason = evaluate_expect_skip("git tag v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_verbose() {
        let outcome = evaluate_expect_outcome("git remote -v");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add() {
        let reason = evaluate_expect_skip("git remote add upstream https://x.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rm_tmp_file() {
        let outcome = evaluate_expect_outcome("rm /tmp/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d() {
        let outcome = evaluate_expect_outcome("git clean -fd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn forked_path() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/forked/repo status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_path() {
        let reason = evaluate_expect_skip("git -C /tmp/sketchy status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_pop() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_reset_hard() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project reset --hard");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_checkout_discard() {
        let outcome =
            evaluate_expect_outcome("git -C /home/user/repos/my-project checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_git_clean_d() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project clean -fd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_status_piped() {
        let outcome = evaluate_expect_outcome("git status | head -5");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_diff_and_status() {
        let outcome = evaluate_expect_outcome("git diff && git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn safe_and_unknown() {
        let reason = evaluate_expect_skip("git status && cargo publish");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn semi_both_safe() {
        let outcome = evaluate_expect_outcome("git status ; git diff");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_safe_and_unknown() {
        let outcome = evaluate_expect_outcome("git status ; cargo publish");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_safe_and_rm() {
        let outcome = evaluate_expect_outcome("git status ; rm -rf /tmp/nothing");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_mixed_with_and() {
        let outcome = evaluate_expect_outcome("git status && git diff ; git log");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn for_loop_echo() {
        let outcome = evaluate_expect_outcome("for f in *.txt; do echo $f; done");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_safe_git() {
        let outcome = evaluate_expect_outcome("for f in *.txt; do git status; done");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_rm() {
        let outcome = evaluate_expect_outcome("for f in *.tmp; do rm $f; done");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn for_loop_safe_and_unknown() {
        let reason = evaluate_expect_skip("for f in *.txt; do git status && cargo publish; done");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn for_loop_unknown() {
        let reason = evaluate_expect_skip("for f in *.txt; do cargo publish; done");
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
