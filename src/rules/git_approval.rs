//! Allow rules for safe, read-only git subcommands.

use crate::prelude::*;

const SAFE_SUBCOMMANDS: &[&str] = &[
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
pub fn git_approval_rules() -> Vec<SimpleRule> {
    let mut rules: Vec<SimpleRule> = SAFE_SUBCOMMANDS
        .iter()
        .map(|sub| {
            SimpleRule::new(
                format!("git {sub}"),
                Outcome::allow(format!("Safe git subcommand: {sub}")),
            )
        })
        .collect();

    // git branch (bare)
    rules.push(SimpleRule {
        prefix: "git branch".to_owned(),
        condition: Some(|cmd| cmd.args.len() == 1),
        outcome: Outcome::allow("Safe git subcommand: branch"),
        ..Default::default()
    });
    // git branch (read-only flags)
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
        rules.push(SimpleRule {
            prefix: "git branch".to_owned(),
            with_option: Some(vec![flag.to_owned()]),
            outcome: Outcome::allow("Safe git subcommand: branch"),
            ..Default::default()
        });
    }

    // git tag (bare)
    rules.push(SimpleRule {
        prefix: "git tag".to_owned(),
        condition: Some(|cmd| cmd.args.len() == 1),
        outcome: Outcome::allow("Safe git subcommand: tag"),
        ..Default::default()
    });
    // git tag (read-only flags)
    for flag in [
        "-l",
        "--list",
        "-n",
        "--contains",
        "--merged",
        "--no-merged",
        "-v",
        "--verify",
    ] {
        rules.push(SimpleRule {
            prefix: "git tag".to_owned(),
            with_option: Some(vec![flag.to_owned()]),
            condition: Some(|cmd| !has_positional_after_tag(cmd)),
            outcome: Outcome::allow("Safe git subcommand: tag"),
            ..Default::default()
        });
    }

    // git remote (read-only subcommands)
    for sub in ["-v", "--verbose", "show", "get-url"] {
        rules.push(SimpleRule::new(
            format!("git remote {sub}"),
            Outcome::allow("Safe git subcommand: remote"),
        ));
    }
    rules.push(SimpleRule {
        prefix: "git remote".to_owned(),
        condition: Some(|cmd| cmd.args.len() == 1),
        outcome: Outcome::allow("Safe git subcommand: remote"),
        ..Default::default()
    });

    // git -C <trusted-path> <safe-subcommand>
    rules.push(SimpleRule {
        prefix: "git -C".to_owned(),
        condition: Some(is_trusted_safe_git),
        outcome: Outcome::allow("Safe git subcommand in trusted path"),
        ..Default::default()
    });

    rules
}

fn has_positional_after_tag(cmd: &SimpleContext) -> bool {
    // args: ["tag", ...rest]
    cmd.args.iter().skip(1).any(|a| !a.starts_with('-'))
}

fn is_trusted_safe_git(cmd: &SimpleContext) -> bool {
    let ga = match parse_git_args(cmd) {
        Some(ga) => ga,
        None => return false,
    };
    let path = ga.path.as_deref().unwrap_or("");
    if classify_path(path) != PathClass::Trusted {
        return false;
    }
    let first = match ga.args.first() {
        Some(a) => a.as_str(),
        None => return false,
    };
    SAFE_SUBCOMMANDS.contains(&first) || matches!(first, "branch" | "tag" | "remote")
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn safe_subcommands_no_path() {
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
            let result = evaluate(&format!("git {sub}"));
            assert!(
                result
                    .as_ref()
                    .is_some_and(|r| r.decision == Decision::Allow),
                "git {sub} should be allowed: {result:?}"
            );
        }
    }

    #[test]
    fn log_with_args() {
        let result = evaluate("git log --oneline -5");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn diff_head_1() {
        let result = evaluate("git diff HEAD~1");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_show() {
        let result = evaluate("git remote show origin");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_get_url() {
        let result = evaluate("git remote get-url origin");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn trusted_path_status() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura status");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn trusted_path_log() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/alnwick log --oneline");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn trusted_subdir_diff() {
        let result = evaluate("git -C /var/mnt/e/Repos/Infrastructure/homelab diff");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn double_quoted_trusted_path() {
        let result = evaluate("git -C \"/var/mnt/e/Repos/Rust/caesura\" status");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn single_quoted_trusted_path() {
        let result = evaluate("git -C '/var/mnt/e/Repos/Rust/caesura' status");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn trailing_slash_trusted_path() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura/ status");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn forked_status_passthrough() {
        assert_eq!(
            evaluate("git -C /var/mnt/e/Repos/Forked/some-repo status"),
            None
        );
    }

    #[test]
    fn forked_log_passthrough() {
        assert_eq!(
            evaluate("git -C /var/mnt/e/Repos/Forked/some-repo log"),
            None
        );
    }

    #[test]
    fn unknown_status_passthrough() {
        assert_eq!(evaluate("git -C /tmp/sketchy-repo status"), None);
    }

    #[test]
    fn unknown_diff_passthrough() {
        assert_eq!(evaluate("git -C /home/other/repo diff"), None);
    }

    #[test]
    fn unsafe_commit_passthrough() {
        assert_eq!(evaluate("git commit -m 'test'"), None);
    }

    #[test]
    fn unsafe_push_passthrough() {
        assert_eq!(evaluate("git push origin main"), None);
    }

    #[test]
    fn unsafe_add_passthrough() {
        assert_eq!(evaluate("git add -A"), None);
    }

    #[test]
    fn unsafe_rebase_passthrough() {
        assert_eq!(evaluate("git rebase main"), None);
    }

    #[test]
    fn unsafe_reset_passthrough() {
        // git reset --hard is Deny via git rules, not passthrough
        let result = evaluate("git reset --hard HEAD~1").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn unsafe_checkout_passthrough() {
        // git checkout -- is Deny via git_checkout rules
        let result = evaluate("git checkout -- file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn unsafe_stash_passthrough() {
        // git stash pop is Deny via git rules
        let result = evaluate("git stash pop").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn unsafe_remote_add_passthrough() {
        assert_eq!(
            evaluate("git remote add upstream https://example.com"),
            None
        );
    }

    #[test]
    fn unsafe_with_c_path_commit_passthrough() {
        assert_eq!(
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura commit -m 'test'"),
            None
        );
    }

    #[test]
    fn unsafe_with_c_path_push_passthrough() {
        assert_eq!(
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura push origin main"),
            None
        );
    }

    #[test]
    fn unsafe_with_c_path_add_passthrough() {
        assert_eq!(
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura add -A"),
            None
        );
    }

    #[test]
    fn non_git_ls_passthrough() {
        // ls is Allow via safe_rules
        let result = evaluate("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn non_git_cargo_passthrough() {
        assert_eq!(evaluate("cargo build"), None);
    }

    #[test]
    fn non_git_cat_passthrough() {
        // cat is Allow via safe_rules
        let result = evaluate("cat file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn options_before_subcommand_passthrough() {
        assert_eq!(evaluate("git --no-pager log"), None);
    }

    #[test]
    fn options_no_pager_diff_passthrough() {
        assert_eq!(evaluate("git --no-pager diff HEAD~1"), None);
    }

    #[test]
    fn options_c_config_passthrough() {
        assert_eq!(evaluate("git -c core.pager= status"), None);
    }

    #[test]
    fn chained_ls_git_passthrough() {
        let result = evaluate("ls && git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn chained_git_git_passthrough() {
        assert_eq!(evaluate("git add file.txt && git commit -m 'test'"), None);
    }

    #[test]
    fn chained_git_c_push_passthrough() {
        assert_eq!(evaluate("git status && git -C /tmp/evil push"), None);
    }

    #[test]
    fn chained_or_passthrough() {
        let result = evaluate("git log || git diff").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn chained_semicolon_passthrough() {
        assert_eq!(evaluate("git status ; git log"), None);
    }

    #[test]
    fn chained_pipe_passthrough() {
        let result = evaluate("git status | head -5").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_c_passthrough() {
        // grep is Allow via safe_rules
        let result = evaluate("grep -C 3 pattern file").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_git_passthrough() {
        let result = evaluate("echo git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_git_c_quoted_passthrough() {
        let result = evaluate("echo 'git -C /path status'").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn unsafe_unknown_commit_passthrough() {
        assert_eq!(evaluate("git -C /tmp/sketchy commit -m 'evil'"), None);
    }

    // Branch flag tests
    #[test]
    fn branch_bare() {
        let result = evaluate("git branch");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_all() {
        let result = evaluate("git branch -a");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_list() {
        let result = evaluate("git branch --list");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_remotes() {
        let result = evaluate("git branch -r");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_verbose() {
        let result = evaluate("git branch -v");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_double_verbose() {
        let result = evaluate("git branch -vv");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_contains() {
        let result = evaluate("git branch --contains");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_merged() {
        let result = evaluate("git branch --merged");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_no_merged() {
        let result = evaluate("git branch --no-merged");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_sort_passthrough() {
        assert_eq!(evaluate("git branch --sort=committerdate"), None);
    }

    #[test]
    fn branch_format_passthrough() {
        assert_eq!(evaluate("git branch --format='%(refname:short)'"), None);
    }

    #[test]
    fn branch_points_at() {
        let result = evaluate("git branch --points-at HEAD");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_combined_read_flags() {
        // -a and -v match read-only flags; --sort= is ignored (not matched)
        let result = evaluate("git branch -a -v --sort=committerdate").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn branch_delete_passthrough() {
        assert_eq!(evaluate("git branch -d old-branch"), None);
    }

    #[test]
    fn branch_force_delete_passthrough() {
        assert_eq!(evaluate("git branch -D old-branch"), None);
    }

    #[test]
    fn branch_move_passthrough() {
        assert_eq!(evaluate("git branch -m old new"), None);
    }

    #[test]
    fn branch_force_move_passthrough() {
        assert_eq!(evaluate("git branch -M old new"), None);
    }

    #[test]
    fn branch_copy_passthrough() {
        assert_eq!(evaluate("git branch -c old new"), None);
    }

    #[test]
    fn branch_long_delete_passthrough() {
        assert_eq!(evaluate("git branch --delete old-branch"), None);
    }

    #[test]
    fn branch_long_move_passthrough() {
        assert_eq!(evaluate("git branch --move old new"), None);
    }

    #[test]
    fn branch_long_copy_passthrough() {
        assert_eq!(evaluate("git branch --copy old new"), None);
    }

    #[test]
    fn branch_set_upstream_passthrough() {
        assert_eq!(evaluate("git branch --set-upstream-to=origin/main"), None);
    }

    #[test]
    fn branch_unset_upstream_passthrough() {
        assert_eq!(evaluate("git branch --unset-upstream"), None);
    }

    // Tag flag tests
    #[test]
    fn tag_bare() {
        let result = evaluate("git tag");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_list() {
        let result = evaluate("git tag -l");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_list_long() {
        let result = evaluate("git tag --list");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_n() {
        let result = evaluate("git tag -n");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_n5() {
        let result = evaluate("git tag -n5");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_contains() {
        let result = evaluate("git tag --contains");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_merged() {
        let result = evaluate("git tag --merged");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_no_merged() {
        let result = evaluate("git tag --no-merged");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_sort_passthrough() {
        assert_eq!(evaluate("git tag --sort=version:refname"), None);
    }

    #[test]
    fn tag_verify() {
        let result = evaluate("git tag -v v1.0");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_verify_long() {
        let result = evaluate("git tag --verify v1.0");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_delete_passthrough() {
        assert_eq!(evaluate("git tag -d v1.0"), None);
    }

    #[test]
    fn tag_annotated_passthrough() {
        assert_eq!(evaluate("git tag -a v1.0 -m 'release'"), None);
    }

    #[test]
    fn tag_signed_passthrough() {
        assert_eq!(evaluate("git tag -s v1.0"), None);
    }

    #[test]
    fn tag_force_passthrough() {
        assert_eq!(evaluate("git tag -f v1.0"), None);
    }

    #[test]
    fn tag_message_passthrough() {
        assert_eq!(evaluate("git tag -m 'release'"), None);
    }

    #[test]
    fn tag_positional_create_passthrough() {
        assert_eq!(evaluate("git tag v1.0"), None);
    }

    #[test]
    fn tag_long_delete_passthrough() {
        assert_eq!(evaluate("git tag --delete v1.0"), None);
    }

    // Remote flag tests
    #[test]
    fn remote_bare() {
        let result = evaluate("git remote");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_verbose() {
        let result = evaluate("git remote -v");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_verbose_long() {
        let result = evaluate("git remote --verbose");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_show_origin() {
        let result = evaluate("git remote show origin");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_get_url_origin() {
        let result = evaluate("git remote get-url origin");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_add_passthrough() {
        assert_eq!(
            evaluate("git remote add upstream https://example.com"),
            None
        );
    }

    #[test]
    fn remote_remove_passthrough() {
        assert_eq!(evaluate("git remote remove upstream"), None);
    }

    #[test]
    fn remote_rename_passthrough() {
        assert_eq!(evaluate("git remote rename origin upstream"), None);
    }

    #[test]
    fn remote_set_url_passthrough() {
        assert_eq!(
            evaluate("git remote set-url origin https://example.com"),
            None
        );
    }

    #[test]
    fn remote_set_head_passthrough() {
        assert_eq!(evaluate("git remote set-head origin main"), None);
    }

    #[test]
    fn remote_set_branches_passthrough() {
        assert_eq!(evaluate("git remote set-branches origin main"), None);
    }

    #[test]
    fn remote_prune_passthrough() {
        assert_eq!(evaluate("git remote prune origin"), None);
    }

    // Branch/tag/remote with -C trusted path
    #[test]
    fn branch_trusted_path() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura branch -a");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tag_trusted_path() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura tag -l");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn remote_trusted_path() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura remote -v");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn branch_forked_path_passthrough() {
        assert_eq!(evaluate("git -C /var/mnt/e/Repos/Forked/repo branch"), None);
    }

    #[test]
    fn branch_delete_with_path() {
        // -d is a write flag but trusted path matches "git -C" rule for branch
        let result =
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura branch -d old").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn stash_with_c_path_denied() {
        let result =
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura stash pop").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn reset_with_c_path_denied() {
        let result =
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura reset --hard").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn checkout_with_c_path_denied() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura checkout -- file.txt")
            .expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn clean_with_c_path_denied() {
        let result =
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura clean -fd").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }
}
