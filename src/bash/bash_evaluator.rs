//! Rule evaluation: matches parsed commands against registered rules.

use crate::prelude::*;

/// Rule engine that evaluates parsed shell commands against registered security rules.
pub struct BashEvaluator {
    /// User settings for path classification and trusted directories.
    settings: Settings,
    /// Registered security rules for matching commands.
    rules: Vec<BashRule>,
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
        let mut all_outcomes = Vec::new();
        let mut has_unmatched = false;
        for simple_context in complete_context.all_commands() {
            let mut outcomes = Vec::new();
            for rule in &self.rules {
                if rule.matches(simple_context, complete_context, &self.settings) {
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

impl BashEvaluator {
    /// Create an evaluator with the given settings.
    pub fn new(settings: Settings) -> Self {
        let mut rules = Vec::new();
        rules.extend(modern_alternative_rules());
        rules.push(rm());
        rules.extend(fd_rules());
        rules.extend(find_rules());
        rules.extend(gh_rules());
        rules.extend(git_deny_rules());
        rules.extend(git_allow_rules());
        rules.extend(git_c_rules());
        rules.extend(insta_rules());
        rules.extend(cd_git_rules());
        rules.extend(chained_push_rules());
        rules.extend(long_python_rules());
        rules.extend(safe_rules());
        Self { settings, rules }
    }
}

#[cfg(test)]
/// Parse and evaluate `command`, expecting a successful [`Outcome`].
pub(crate) fn evaluate_expect_outcome(command: &str) -> Outcome {
    let _logger = init_test_logger();
    BashEvaluator::new(Settings::mock())
        .evaluate_str(command)
        .expect("command should produce an outcome")
}

#[cfg(test)]
/// Parse and evaluate `command`, expecting a [`SkipReason`].
#[expect(clippy::panic, reason = "test helper")]
pub(crate) fn evaluate_expect_skip(command: &str) -> SkipReason {
    let _logger = init_test_logger();
    match BashEvaluator::new(Settings::mock())
        .evaluate_str(command)
        .expect_err("command should not succeed")
        .current_context()
    {
        ParseError::Skip(reason) => *reason,
        other => panic!("expected Skip, got {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_git_allowed() {
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rm_denied() {
        let outcome = evaluate_expect_outcome("rm -rf /tmp/nothing");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn stash_pop_denied() {
        let outcome = evaluate_expect_outcome("git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn reset_hard_denied() {
        let outcome = evaluate_expect_outcome("git reset --hard");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn checkout_discard_denied() {
        let outcome = evaluate_expect_outcome("git checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn chained_push_denied() {
        let outcome = evaluate_expect_outcome("git commit -m 'msg' && git push");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn echo_separator_passthrough() {
        let reason = evaluate_expect_skip("cmd && echo \"---\"");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn find_delete_denied() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn insta_heredoc_denied() {
        let outcome = evaluate_expect_outcome("cargo insta review <<EOF\na\nEOF");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_git_denied() {
        let outcome = evaluate_expect_outcome("cd /path && git status");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn plain_ls_allowed() {
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn plain_cargo_passthrough() {
        let reason = evaluate_expect_skip("cargo build");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn standalone_push_passthrough() {
        let reason = evaluate_expect_skip("git push");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_read_allowed() {
        let outcome = evaluate_expect_outcome("git branch -a");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_write_passthrough() {
        let reason = evaluate_expect_skip("git branch -d old");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_read_allowed() {
        let outcome = evaluate_expect_outcome("git tag -l");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_create_passthrough() {
        let reason = evaluate_expect_skip("git tag v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_verbose_allowed() {
        let outcome = evaluate_expect_outcome("git remote -v");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add_passthrough() {
        let reason = evaluate_expect_skip("git remote add upstream https://x.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn tmp_rm_denied() {
        let outcome = evaluate_expect_outcome("rm /tmp/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_denied() {
        let outcome = evaluate_expect_outcome("git clean -fd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn forked_path_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/forked/repo status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_path_passthrough() {
        let reason = evaluate_expect_skip("git -C /tmp/sketchy status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_pop_denied() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_reset_hard_denied() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project reset --hard");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_checkout_discard_denied() {
        let outcome =
            evaluate_expect_outcome("git -C /home/user/repos/my-project checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_git_clean_d_denied() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project clean -fd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_status_piped_allowed() {
        let outcome = evaluate_expect_outcome("git status | head -5");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_diff_and_status_allowed() {
        let outcome = evaluate_expect_outcome("git diff && git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn safe_and_unknown_passthrough() {
        let reason = evaluate_expect_skip("git status && cargo build");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn semi_both_allowed() {
        let outcome = evaluate_expect_outcome("git status ; git diff");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn semi_allow_and_unknown_passthrough() {
        let reason = evaluate_expect_skip("git status ; cargo build");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn semi_allow_and_deny() {
        let outcome = evaluate_expect_outcome("git status ; rm -rf /tmp/nothing");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn semi_mixed_with_and() {
        let outcome = evaluate_expect_outcome("git status && git diff ; git log");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_echo_allowed() {
        let outcome = evaluate_expect_outcome("for f in *.txt; do echo $f; done");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_safe_git_allowed() {
        let outcome = evaluate_expect_outcome("for f in *.txt; do git status; done");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_deny_denied() {
        let outcome = evaluate_expect_outcome("for f in *.tmp; do rm $f; done");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn for_loop_allow_and_unknown_passthrough() {
        let reason = evaluate_expect_skip("for f in *.txt; do git status && cargo build; done");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn for_loop_unknown_passthrough() {
        let reason = evaluate_expect_skip("for f in *.txt; do cargo build; done");
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
