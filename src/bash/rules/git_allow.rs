//! Allow rules for safe, read-only git subcommands.

use crate::prelude::*;

/// Git subcommands considered safe (read-only or low-risk).
pub(crate) const SAFE_SUBCOMMANDS: &[&str] = &[
    "check-ignore",
    "describe",
    "diff",
    "fetch",
    "log",
    "ls-tree",
    "merge-base",
    "mv",
    "rev-parse",
    "rm",
    "show",
    "status",
];

/// Allow read-only git subcommands, including trusted-path variants via `git -C`.
pub fn git_allow_rules() -> Vec<SimpleRule> {
    let mut rules: Vec<SimpleRule> = SAFE_SUBCOMMANDS
        .iter()
        .map(|sub| {
            SimpleRule::new(
                format!("git_{sub}").replace('-', "_"),
                format!("git {sub}"),
                Outcome::allow(format!("Safe git subcommand: {sub}")),
            )
        })
        .collect();

    rules.push(git_branch__bare());
    for flag in [
        "-a",
        "--all",
        "-l",
        "--list",
        "-r",
        "--remotes",
        "-v",
        "--verbose",
        "-vv",
        "--contains",
        "--merged",
        "--no-merged",
        "--points-at",
    ] {
        let flag_id = flag.trim_start_matches('-').replace('-', "_");
        rules.push(SimpleRule {
            id: format!("git_branch_{flag_id}"),
            prefix: "git branch".to_owned(),
            with_any: Some(vec![Arg::new(flag)]),
            outcome: Outcome::allow("Safe git subcommand: branch"),
            ..Default::default()
        });
    }

    rules.push(git_tag__bare());
    rules.push(SimpleRule {
        id: "git_tag__read_only".to_owned(),
        prefix: "git tag".to_owned(),
        with_any: Some(
            ["-l", "--list", "--contains", "--merged", "--no-merged"]
                .into_iter()
                .map(Arg::new)
                .collect(),
        ),
        outcome: Outcome::allow("Safe git subcommand: tag"),
        ..Default::default()
    });

    for sub in ["-v", "--verbose", "show", "get-url"] {
        let sub_id = sub.trim_start_matches('-').replace('-', "_");
        rules.push(SimpleRule::new(
            format!("git_remote_{sub_id}"),
            format!("git remote {sub}"),
            Outcome::allow("Safe git subcommand: remote"),
        ));
    }
    rules.push(git_remote__bare());

    rules
}

/// Allow bare `git branch` (no arguments).
fn git_branch__bare() -> SimpleRule {
    SimpleRule {
        id: "git_branch__bare".to_owned(),
        prefix: "git branch".to_owned(),
        condition: Some(|cmd, _, _| cmd.args.len() == 1),
        outcome: Outcome::allow("Safe git subcommand: branch"),
        ..Default::default()
    }
}

/// Allow bare `git tag` (no arguments).
fn git_tag__bare() -> SimpleRule {
    SimpleRule {
        id: "git_tag__bare".to_owned(),
        prefix: "git tag".to_owned(),
        condition: Some(|cmd, _, _| cmd.args.len() == 1),
        outcome: Outcome::allow("Safe git subcommand: tag"),
        ..Default::default()
    }
}

/// Allow bare `git remote` (no arguments).
fn git_remote__bare() -> SimpleRule {
    SimpleRule {
        id: "git_remote__bare".to_owned(),
        prefix: "git remote".to_owned(),
        condition: Some(|cmd, _, _| cmd.args.len() == 1),
        outcome: Outcome::allow("Safe git subcommand: remote"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn _git_safe_subcommands() {
        for sub in [
            "check-ignore",
            "describe",
            "diff",
            "fetch",
            "log",
            "ls-tree",
            "merge-base",
            "mv",
            "rev-parse",
            "rm",
            "show",
            "status",
        ] {
            let outcome = evaluate_expect_outcome(&format!("git {sub}"));
            assert_eq!(outcome.decision, Decision::Allow, "git {sub}");
        }
    }

    #[test]
    fn _git_log__args() {
        let outcome = evaluate_expect_outcome("git log --oneline -5");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_diff__head_1() {
        let outcome = evaluate_expect_outcome("git diff HEAD~1");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_show() {
        let outcome = evaluate_expect_outcome("git remote show origin");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_get_url() {
        let outcome = evaluate_expect_outcome("git remote get-url origin");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_commit() {
        let reason = evaluate_expect_skip("git commit -m 'test'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_push() {
        let reason = evaluate_expect_skip("git push origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_add() {
        let reason = evaluate_expect_skip("git add -A");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_rebase() {
        let reason = evaluate_expect_skip("git rebase main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_reset_hard() {
        // git reset --hard is Deny via git rules, not passthrough
        let outcome = evaluate_expect_outcome("git reset --hard HEAD~1");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _git_checkout_discard() {
        // git checkout -- is Deny via git_checkout rules
        let outcome = evaluate_expect_outcome("git checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _git_stash_pop() {
        // git stash pop is Deny via git rules
        let outcome = evaluate_expect_outcome("git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _git_remote_add() {
        let reason = evaluate_expect_skip("git remote add upstream https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _ls() {
        // ls is Allow via safe_rules
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _cargo() {
        let reason = evaluate_expect_skip("cargo build");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _cat() {
        // cat is Allow via safe_rules
        let outcome = evaluate_expect_outcome("cat file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git__no_pager_log() {
        let reason = evaluate_expect_skip("git --no-pager log");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git__no_pager_diff() {
        let reason = evaluate_expect_skip("git --no-pager diff HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git__c_config() {
        let reason = evaluate_expect_skip("git -c core.pager= status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _ls_git_status() {
        let outcome = evaluate_expect_outcome("ls && git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_add_commit() {
        let reason = evaluate_expect_skip("git add file.txt && git commit -m 'test'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_log_or_diff() {
        let outcome = evaluate_expect_outcome("git log || git diff");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_status_semicolon_log() {
        let outcome = evaluate_expect_outcome("git status ; git log");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_status_pipe() {
        let outcome = evaluate_expect_outcome("git status | head -5");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _grep_c() {
        // grep is Allow via safe_rules
        let outcome = evaluate_expect_outcome("grep -C 3 pattern file");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _echo_git() {
        let outcome = evaluate_expect_outcome("echo git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    // Branch flag tests
    #[test]
    fn _git_branch__bare() {
        let outcome = evaluate_expect_outcome("git branch");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_a() {
        let outcome = evaluate_expect_outcome("git branch -a");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_list() {
        let outcome = evaluate_expect_outcome("git branch --list");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_r() {
        let outcome = evaluate_expect_outcome("git branch -r");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_v() {
        let outcome = evaluate_expect_outcome("git branch -v");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_vv() {
        let outcome = evaluate_expect_outcome("git branch -vv");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_contains() {
        let outcome = evaluate_expect_outcome("git branch --contains");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_merged() {
        let outcome = evaluate_expect_outcome("git branch --merged");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_no_merged() {
        let outcome = evaluate_expect_outcome("git branch --no-merged");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch_sort() {
        let reason = evaluate_expect_skip("git branch --sort=committerdate");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_format() {
        let reason = evaluate_expect_skip("git branch --format='%(refname:short)'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_points_at() {
        let outcome = evaluate_expect_outcome("git branch --points-at HEAD");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_branch__combined() {
        // -a and -v match read-only flags; --sort= is ignored (not matched)
        let outcome = evaluate_expect_outcome("git branch -a -v --sort=committerdate");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_branch_d() {
        let reason = evaluate_expect_skip("git branch -d old-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_cap_d() {
        let reason = evaluate_expect_skip("git branch -D old-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_m() {
        let reason = evaluate_expect_skip("git branch -m old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_cap_m() {
        let reason = evaluate_expect_skip("git branch -M old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_c() {
        let reason = evaluate_expect_skip("git branch -c old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_delete() {
        let reason = evaluate_expect_skip("git branch --delete old-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_move() {
        let reason = evaluate_expect_skip("git branch --move old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_copy() {
        let reason = evaluate_expect_skip("git branch --copy old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_set_upstream() {
        let reason = evaluate_expect_skip("git branch --set-upstream-to=origin/main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_branch_unset_upstream() {
        let reason = evaluate_expect_skip("git branch --unset-upstream");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    // Tag flag tests
    #[test]
    fn _git_tag__bare() {
        let outcome = evaluate_expect_outcome("git tag");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_tag_l() {
        let outcome = evaluate_expect_outcome("git tag -l");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_tag_list() {
        let outcome = evaluate_expect_outcome("git tag --list");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_tag_n() {
        let reason = evaluate_expect_skip("git tag -n");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_n5() {
        let reason = evaluate_expect_skip("git tag -n5");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_contains() {
        let outcome = evaluate_expect_outcome("git tag --contains");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_tag_contains_commit() {
        let outcome = evaluate_expect_outcome("git tag --contains f4ce32b");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_tag_merged() {
        let outcome = evaluate_expect_outcome("git tag --merged");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_tag_merged_commit() {
        let outcome = evaluate_expect_outcome("git tag --merged main");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_tag_no_merged() {
        let outcome = evaluate_expect_outcome("git tag --no-merged");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_tag_no_merged_commit() {
        let outcome = evaluate_expect_outcome("git tag --no-merged main");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_tag_list_pattern() {
        let outcome = evaluate_expect_outcome("git tag -l 'v1.*'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_tag_sort() {
        let reason = evaluate_expect_skip("git tag --sort=version:refname");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_verify() {
        let reason = evaluate_expect_skip("git tag -v v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_verify_long() {
        let reason = evaluate_expect_skip("git tag --verify v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_d() {
        let reason = evaluate_expect_skip("git tag -d v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_a() {
        let reason = evaluate_expect_skip("git tag -a v1.0 -m 'release'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_s() {
        let reason = evaluate_expect_skip("git tag -s v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_f() {
        let reason = evaluate_expect_skip("git tag -f v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_m() {
        let reason = evaluate_expect_skip("git tag -m 'release'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_positional() {
        let reason = evaluate_expect_skip("git tag v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_tag_delete() {
        let reason = evaluate_expect_skip("git tag --delete v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    // Remote flag tests
    #[test]
    fn _git_remote__bare() {
        let outcome = evaluate_expect_outcome("git remote");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_v() {
        let outcome = evaluate_expect_outcome("git remote -v");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_verbose() {
        let outcome = evaluate_expect_outcome("git remote --verbose");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_show_origin() {
        let outcome = evaluate_expect_outcome("git remote show origin");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_get_url_origin() {
        let outcome = evaluate_expect_outcome("git remote get-url origin");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_remote_add_upstream() {
        let reason = evaluate_expect_skip("git remote add upstream https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_remote_remove() {
        let reason = evaluate_expect_skip("git remote remove upstream");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_remote_rename() {
        let reason = evaluate_expect_skip("git remote rename origin upstream");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_remote_set_url() {
        let reason = evaluate_expect_skip("git remote set-url origin https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_remote_set_head() {
        let reason = evaluate_expect_skip("git remote set-head origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_remote_set_branches() {
        let reason = evaluate_expect_skip("git remote set-branches origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_remote_prune() {
        let reason = evaluate_expect_skip("git remote prune origin");
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
