//! Deny and allow rules for `git -C <trusted-path>` commands.

use super::git_allow::git_allow_rules;
use super::git_deny::git_deny_rules;
use crate::prelude::*;

/// Deny and allow rules for `git -C <trusted-path>` commands.
pub fn git_c_rules() -> Vec<BashRule> {
    vec![git_c__deny_destructive(), git_c__allow_trusted()]
}

/// Deny destructive `git -C` operations.
fn git_c__deny_destructive() -> BashRule {
    BashRule {
        id: "git_c__deny_destructive".to_owned(),
        command: "git -C".to_owned(),
        condition: Some(deny_git_c),
        outcome: Outcome::deny("Destructive `git -C` operation is blocked"),
        ..Default::default()
    }
}

/// Allow safe `git -C` in trusted paths.
fn git_c__allow_trusted() -> BashRule {
    BashRule {
        id: "git_c__allow_trusted".to_owned(),
        command: "git -C".to_owned(),
        condition: Some(allow_git_c),
        outcome: Outcome::allow("Safe `git -C` in trusted path"),
        ..Default::default()
    }
}

fn guard(context: &SimpleContext) -> bool {
    context.name == "git"
        && context.args.first().is_some_and(|arg| arg == "-C")
        && context.args.len() > 2
}

fn get_context_without_c(context: &SimpleContext) -> SimpleContext {
    SimpleContext {
        name: "git".to_owned(),
        args: context.args.get(2..).unwrap_or_default().to_vec(),
        has_heredoc: context.has_heredoc,
        contains_substitution: context.contains_substitution,
        nesting: context.nesting.clone(),
        env_vars: context.env_vars.clone(),
    }
}

#[expect(
    clippy::indexing_slicing,
    reason = "guard() ensures args.len() > 2, so index 1 is safe"
)]
fn is_c_path_trusted(ctx: &BashRuleContext) -> bool {
    let path = unquote_str(&ctx.simple.args[1]);
    if let Some(is_allowed) = ctx.paths.is_match(&path, &ctx.settings.git.paths) {
        trace!(is_allowed, "Matched");
        is_allowed
    } else {
        trace!("No match");
        false
    }
}

fn deny_git_c(ctx: &BashRuleContext) -> bool {
    if !guard(ctx.simple) {
        return false;
    }
    let new_simple = get_context_without_c(ctx.simple);
    let inner = BashRuleContext {
        simple: &new_simple,
        complete: ctx.complete,
        settings: ctx.settings,
        paths: ctx.paths,
    };
    git_deny_rules().iter().any(|r| r.matches(&inner))
}

fn allow_git_c(ctx: &BashRuleContext) -> bool {
    if !guard(ctx.simple) || !is_c_path_trusted(ctx) {
        return false;
    }
    let new_simple = get_context_without_c(ctx.simple);
    let inner = BashRuleContext {
        simple: &new_simple,
        complete: ctx.complete,
        settings: ctx.settings,
        paths: ctx.paths,
    };
    git_allow_rules().iter().any(|r| r.matches(&inner))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn negation_overrides_earlier_trust() {
        let settings = Settings::with_git(&["/a/b/**", "!/a/b/forked/**"]);
        let result =
            eval_rules_with_settings(git_c_rules(), "git -C /a/b/forked/repo status", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn re_include_after_negation() {
        let settings = Settings::with_git(&[
            "/home/user/repos/**",
            "!/home/user/repos/forked/**",
            "/home/user/repos/forked/this",
        ]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/forked/this status",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn re_include_does_not_affect_other_forked() {
        let settings = Settings::with_git(&[
            "/home/user/repos/**",
            "!/home/user/repos/forked/**",
            "/home/user/repos/forked/this",
        ]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/forked/other status",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn no_patterns() {
        let settings = Settings::with_git(&[]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/foo status",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn single_trust_pattern() {
        let settings = Settings::with_git(&["/home/user/repos/**"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/foo status",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn single_negation_only() {
        let settings = Settings::with_git(&["!/home/user/repos/**"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/foo status",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn path_outside_pattern() {
        let settings = Settings::with_git(&["/home/user/repos/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /tmp/other status", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn last_match_wins_trust_after_negate() {
        let settings = Settings::with_git(&["!/a/**", "/a/b/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/b/repo status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn last_match_wins_negate_after_trust() {
        let settings = Settings::with_git(&["/a/**", "!/a/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/repo status", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn later_trust_overrides_earlier_negation() {
        let settings = Settings::with_git(&["!/a/**", "/a/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/repo status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn three_layer_nesting() {
        let settings = Settings::with_git(&["/a/**", "!/a/b/**", "/a/b/c/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/b/c/repo status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn three_layer_middle_excluded() {
        let settings = Settings::with_git(&["/a/**", "!/a/b/**", "/a/b/c/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/b/other status", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn three_layer_top_still_trusted() {
        let settings = Settings::with_git(&["/a/**", "!/a/b/**", "/a/b/c/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/other status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn exact_path_trust() {
        let settings = Settings::with_git(&["/home/user/repos/exact"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/exact status",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn exact_path_negation() {
        let settings = Settings::with_git(&["/home/user/repos/**", "!/home/user/repos/banned"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C /home/user/repos/banned status",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn non_matching_negation_is_harmless() {
        let settings = Settings::with_git(&["/a/**", "!/b/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/repo status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn duplicate_trust_patterns() {
        let settings = Settings::with_git(&["/a/**", "/a/**"]);
        let result = eval_rules_with_settings(git_c_rules(), "git -C /a/repo status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn four_layer_alternating() {
        let settings = Settings::with_git(&["/a/**", "!/a/b/**", "/a/b/c/**", "!/a/b/c/d/**"]);
        let result =
            eval_rules_with_settings(git_c_rules(), "git -C /a/b/c/d/repo status", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn four_layer_third_level_trusted() {
        let settings = Settings::with_git(&["/a/**", "!/a/b/**", "/a/b/c/**", "!/a/b/c/d/**"]);
        let result =
            eval_rules_with_settings(git_c_rules(), "git -C /a/b/c/other status", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_status() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/my-project status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_log() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project log --oneline",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_subdir_diff() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/foo/bar diff");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn double_quoted_trusted_path() {
        let result = eval_rules(
            git_c_rules(),
            "git -C \"/home/user/repos/my-project\" status",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn single_quoted_trusted_path() {
        let result = eval_rules(git_c_rules(), "git -C '/home/user/repos/my-project' status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trailing_slash_trusted_path() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/my-project/ status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn forked_status() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/forked/some-repo status",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn forked_log() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/forked/some-repo log",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_status() {
        let result = eval_rules(git_c_rules(), "git -C /tmp/sketchy-repo status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_diff() {
        let result = eval_rules(git_c_rules(), "git -C /home/other/repo diff");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_commit() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project commit -m 'test'",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_push() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project push origin main",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_add() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/my-project add -A");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_commit() {
        let result = eval_rules(git_c_rules(), "git -C /tmp/sketchy commit -m 'evil'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn branch_trusted_path() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project branch -a",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn tag_trusted_path() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/my-project tag -l");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn remote_trusted_path() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project remote -v",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn branch_forked_path() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/forked/repo branch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn branch_delete_with_path() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project branch -d old",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_reset_hard() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project reset --hard",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_reset_hard_head() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project reset --hard HEAD~1",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_quoted_reset_hard() {
        let result = eval_rules(
            git_c_rules(),
            "git -C \"/home/user/repos/my-project\" reset --hard",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_reset_soft() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project reset --soft HEAD~1",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_pop() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project stash pop",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_stash_drop() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project stash drop",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_quoted_stash_pop() {
        let result = eval_rules(
            git_c_rules(),
            "git -C \"/home/user/repos/my-project\" stash pop",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_stash_apply() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project stash apply",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_bare() {
        let result = eval_rules(git_c_rules(), "git -C /home/user/repos/my-project stash");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_checkout_discard_file() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project checkout -- file.txt",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_checkout_head_file() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project checkout HEAD -- file.txt",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_quoted_checkout_discard() {
        let result = eval_rules(
            git_c_rules(),
            "git -C \"/home/user/repos/my-project\" checkout -- .",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_checkout_branch() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project checkout main",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_clean_fd() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project clean -fd",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_clean_fxd() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project clean -fxd",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_quoted_clean_fd() {
        let result = eval_rules(
            git_c_rules(),
            "git -C \"/home/user/repos/my-project\" clean -fd",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn c_path_clean_f_with_file() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/my-project clean -f file.txt",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn forked_reset_hard() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/forked/repo reset --hard",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn forked_stash_pop() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/forked/repo stash pop",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn forked_clean_fd() {
        let result = eval_rules(
            git_c_rules(),
            "git -C /home/user/repos/forked/repo clean -fd",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unknown_reset_hard() {
        let result = eval_rules(git_c_rules(), "git -C /tmp/sketchy reset --hard");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unknown_stash_pop() {
        let result = eval_rules(git_c_rules(), "git -C /tmp/sketchy stash pop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unknown_stash_clear() {
        let result = eval_rules(git_c_rules(), "git -C /tmp/repo stash clear");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unknown_clean_fd() {
        let result = eval_rules(git_c_rules(), "git -C /tmp/sketchy clean -fd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn chained_git_c_push() {
        let result = eval_rules(git_c_rules(), "git status && git -C /tmp/evil push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_git_c_quoted() {
        let result = eval_rules(git_c_rules(), "echo 'git -C /path status'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Tilde path in `git -C` matches a tilde settings pattern.
    #[test]
    fn tilde_path_trusted() {
        let settings = Settings::with_git(&["~/.config/worktrees/**"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C ~/.config/worktrees/my-project status",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Tilde path in `git -C` matches an absolute settings pattern.
    #[test]
    fn tilde_path_absolute_pattern() {
        let settings = Settings::with_git(&["/home/user/.config/worktrees/**"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C ~/.config/worktrees/my-project log --oneline",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Tilde path in `git -C` that doesn't match any pattern passes through.
    #[test]
    fn tilde_path_untrusted() {
        let settings = Settings::with_git(&["~/.config/worktrees/**"]);
        let result =
            eval_rules_with_settings(git_c_rules(), "git -C ~/.other/repo status", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Destructive operation with tilde path is still denied.
    #[test]
    fn tilde_path_destructive() {
        let settings = Settings::with_git(&["~/.config/worktrees/**"]);
        let result = eval_rules_with_settings(
            git_c_rules(),
            "git -C ~/.config/worktrees/my-project reset --hard",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
