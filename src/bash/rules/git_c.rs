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
        prefix: "git -C".to_owned(),
        condition: Some(deny_git_c),
        outcome: Outcome::deny("Destructive git operation with -C"),
        ..Default::default()
    }
}

/// Allow safe `git -C` in trusted paths.
fn git_c__allow_trusted() -> BashRule {
    BashRule {
        id: "git_c__allow_trusted".to_owned(),
        prefix: "git -C".to_owned(),
        condition: Some(allow_git_c),
        outcome: Outcome::allow("Safe git subcommand in trusted path"),
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
    }
}

#[expect(
    clippy::indexing_slicing,
    reason = "guard() ensures args.len() > 2, so index 1 is safe"
)]
fn is_c_path_trusted(context: &SimpleContext, settings: &Settings) -> bool {
    let path = unquote_str(&context.args[1]);
    !settings
        .git
        .untrusted_dirs
        .iter()
        .any(|d| path.starts_with(d.as_str()))
        && settings
            .git
            .trusted_dirs
            .iter()
            .any(|d| path.starts_with(d.as_str()))
}

fn deny_git_c(context: &SimpleContext, complete: &CompleteContext, settings: &Settings) -> bool {
    if !guard(context) {
        return false;
    }
    let new_context = get_context_without_c(context);
    git_deny_rules()
        .iter()
        .any(|r| r.matches(&new_context, complete, settings))
}

fn allow_git_c(context: &SimpleContext, complete: &CompleteContext, settings: &Settings) -> bool {
    if !guard(context) || !is_c_path_trusted(context, settings) {
        return false;
    }
    let new_context = get_context_without_c(context);
    git_allow_rules()
        .iter()
        .any(|r| r.matches(&new_context, complete, settings))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn trusted_path_status() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn trusted_path_log() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project log --oneline");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn trusted_subdir_diff() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/foo/bar diff");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn double_quoted_trusted_path() {
        let outcome = evaluate_expect_outcome("git -C \"/home/user/repos/my-project\" status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn single_quoted_trusted_path() {
        let outcome = evaluate_expect_outcome("git -C '/home/user/repos/my-project' status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn trailing_slash_trusted_path() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project/ status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn forked_status_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/forked/some-repo status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn forked_log_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/forked/some-repo log");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_status_passthrough() {
        let reason = evaluate_expect_skip("git -C /tmp/sketchy-repo status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unknown_diff_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/other/repo diff");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unsafe_with_c_path_commit_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project commit -m 'test'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unsafe_with_c_path_push_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project push origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unsafe_with_c_path_add_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project add -A");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unsafe_unknown_commit_passthrough() {
        let reason = evaluate_expect_skip("git -C /tmp/sketchy commit -m 'evil'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn branch_trusted_path() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project branch -a");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn tag_trusted_path() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project tag -l");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn remote_trusted_path() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project remote -v");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn branch_forked_path_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/forked/repo branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn branch_delete_with_path_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project branch -d old");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    // === deny: trusted path ===

    #[test]
    fn c_path_reset_hard() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project reset --hard");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_reset_hard_head() {
        let outcome =
            evaluate_expect_outcome("git -C /home/user/repos/my-project reset --hard HEAD~1");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_quoted_reset_hard() {
        let outcome =
            evaluate_expect_outcome("git -C \"/home/user/repos/my-project\" reset --hard");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_reset_soft_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project reset --soft HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_pop() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_stash_drop() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project stash drop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_quoted_stash_pop() {
        let outcome = evaluate_expect_outcome("git -C \"/home/user/repos/my-project\" stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_stash_apply_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project stash apply");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_stash_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project stash");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_checkout_denied() {
        let outcome =
            evaluate_expect_outcome("git -C /home/user/repos/my-project checkout -- file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_checkout_head_file() {
        let outcome =
            evaluate_expect_outcome("git -C /home/user/repos/my-project checkout HEAD -- file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_quoted_checkout_discard() {
        let outcome =
            evaluate_expect_outcome("git -C \"/home/user/repos/my-project\" checkout -- .");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_checkout_branch_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project checkout main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn c_path_clean_fd() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project clean -fd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_clean_fxd() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/my-project clean -fxd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_quoted_clean_fd() {
        let outcome = evaluate_expect_outcome("git -C \"/home/user/repos/my-project\" clean -fd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn c_path_clean_f_passthrough() {
        let reason = evaluate_expect_skip("git -C /home/user/repos/my-project clean -f file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    // === deny: forked path ===

    #[test]
    fn forked_reset_hard_denied() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/forked/repo reset --hard");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn forked_stash_pop_denied() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/forked/repo stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn forked_clean_fd_denied() {
        let outcome = evaluate_expect_outcome("git -C /home/user/repos/forked/repo clean -fd");
        assert_yaml_snapshot!(outcome);
    }

    // === deny: unknown path ===

    #[test]
    fn unknown_reset_hard_denied() {
        let outcome = evaluate_expect_outcome("git -C /tmp/sketchy reset --hard");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn unknown_stash_pop_denied() {
        let outcome = evaluate_expect_outcome("git -C /tmp/sketchy stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn unknown_stash_clear_denied() {
        let outcome = evaluate_expect_outcome("git -C /tmp/repo stash clear");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn unknown_clean_fd_denied() {
        let outcome = evaluate_expect_outcome("git -C /tmp/sketchy clean -fd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn chained_git_c_push_passthrough() {
        let reason = evaluate_expect_skip("git status && git -C /tmp/evil push");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn echo_git_c_quoted_passthrough() {
        let outcome = evaluate_expect_outcome("echo 'git -C /path status'");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
