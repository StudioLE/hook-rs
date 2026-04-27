//! Allow rules for safe git subcommands.

use crate::prelude::*;

/// Git subcommands that only read (no filesystem or `.git/` writes).
pub(crate) const READ_ONLY_SUBCOMMANDS: &[&str] = &[
    "blame",
    "check-ignore",
    "describe",
    "diff",
    "grep",
    "log",
    "ls-tree",
    "merge-base",
    "patch-id",
    "rev-parse",
    "show",
    "status",
];

/// Git subcommands that write but are considered safe.
///
/// - `fetch` writes refs and pack files under `.git/`
/// - `mv` renames working tree files
/// - `rm` deletes working tree files
pub(crate) const SAFE_WRITE_SUBCOMMANDS: &[&str] = &["fetch", "mv", "rm"];

/// Allow safe git subcommands, including trusted-path variants via `git -C`.
pub fn git_allow_rules() -> Vec<BashRule> {
    let mut rules = git_read_only_subcommands();
    rules.extend(git_safe_write_subcommands());
    rules.push(git_branch__bare());
    rules.extend(git_branch__read_only());
    rules.push(git_tag__bare());
    rules.push(git_tag__read_only());
    rules.extend(git_remote__read_only());
    rules.push(git_remote__bare());
    rules.extend(git_worktree__read_only());
    rules.push(git_config_list());
    rules.push(git_config_get());
    rules.push(git_config__read_only_flags());
    rules
}

/// Allow read-only git subcommands.
fn git_read_only_subcommands() -> Vec<BashRule> {
    READ_ONLY_SUBCOMMANDS
        .iter()
        .map(|sub| {
            BashRule::new(
                format!("git_{sub}").replace('-', "_"),
                format!("git {sub}"),
                Outcome::allow(format!("Read-only `git {sub}`")),
            )
        })
        .collect()
}

/// Allow git subcommands that write but are considered safe.
fn git_safe_write_subcommands() -> Vec<BashRule> {
    SAFE_WRITE_SUBCOMMANDS
        .iter()
        .map(|sub| {
            BashRule::new(
                format!("git_{sub}"),
                format!("git {sub}"),
                Outcome::allow(format!("Safe `git {sub}`")),
            )
        })
        .collect()
}

/// Allow bare `git branch` (no arguments).
fn git_branch__bare() -> BashRule {
    BashRule {
        id: "git_branch__bare".to_owned(),
        command: "git branch".to_owned(),
        condition: Some(|simple, _, _| simple.args.len() == 1),
        outcome: Outcome::allow("Read-only `git branch`"),
        ..Default::default()
    }
}

/// Allow read-only `git branch` flags.
fn git_branch__read_only() -> Vec<BashRule> {
    [
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
        "--show-current",
    ]
    .into_iter()
    .map(|flag| {
        let flag_id = flag.trim_start_matches('-').replace('-', "_");
        BashRule {
            id: format!("git_branch_{flag_id}"),
            command: "git branch".to_owned(),
            with_any: Some(vec![Arg::new(flag)]),
            outcome: Outcome::allow("Read-only `git branch`"),
            ..Default::default()
        }
    })
    .collect()
}

/// Allow bare `git tag` (no arguments).
fn git_tag__bare() -> BashRule {
    BashRule {
        id: "git_tag__bare".to_owned(),
        command: "git tag".to_owned(),
        condition: Some(|simple, _, _| simple.args.len() == 1),
        outcome: Outcome::allow("Read-only `git tag`"),
        ..Default::default()
    }
}

/// Allow read-only `git tag` flags like `--list`, `--sort`, and `--verify`.
fn git_tag__read_only() -> BashRule {
    BashRule {
        id: "git_tag__read_only".to_owned(),
        command: "git tag".to_owned(),
        with_any: Some(vec![
            Arg::new("-l"),
            Arg::new("--list"),
            Arg::new("-v"),
            Arg::new("--verify"),
            Arg::new("--contains"),
            Arg::new("--merged"),
            Arg::new("--no-merged"),
            Arg::new("--sort"),
        ]),
        outcome: Outcome::allow("Read-only `git tag`"),
        ..Default::default()
    }
}

/// Allow read-only `git remote` subcommands.
fn git_remote__read_only() -> Vec<BashRule> {
    ["-v", "--verbose", "show", "get-url"]
        .into_iter()
        .map(|sub| {
            let sub_id = sub.trim_start_matches('-').replace('-', "_");
            BashRule::new(
                format!("git_remote_{sub_id}"),
                format!("git remote {sub}"),
                Outcome::allow("Read-only `git remote`"),
            )
        })
        .collect()
}

/// Allow read-only `git worktree` subcommands.
fn git_worktree__read_only() -> Vec<BashRule> {
    ["list"]
        .into_iter()
        .map(|sub| {
            BashRule::new(
                format!("git_worktree_{sub}"),
                format!("git worktree {sub}"),
                Outcome::allow("Read-only `git worktree`"),
            )
        })
        .collect()
}

/// Allow `git config list`.
fn git_config_list() -> BashRule {
    BashRule::new(
        "git_config_list",
        "git config list",
        Outcome::allow("Read-only `git config`"),
    )
}

/// Allow `git config get`.
fn git_config_get() -> BashRule {
    BashRule::new(
        "git_config_get",
        "git config get",
        Outcome::allow("Read-only `git config`"),
    )
}

/// Allow read-only `git config` flag forms (`--get`, `--list`, etc.).
fn git_config__read_only_flags() -> BashRule {
    BashRule {
        id: "git_config__read_only_flags".to_owned(),
        command: "git config".to_owned(),
        with_any: Some(vec![
            Arg::new("-l"),
            Arg::new("--list"),
            Arg::new("--get"),
            Arg::new("--get-all"),
            Arg::new("--get-regexp"),
            Arg::new("--get-urlmatch"),
            Arg::new("--get-color"),
            Arg::new("--get-colorbool"),
        ]),
        outcome: Outcome::allow("Read-only `git config`"),
        ..Default::default()
    }
}

/// Allow bare `git remote` (no arguments).
fn git_remote__bare() -> BashRule {
    BashRule {
        id: "git_remote__bare".to_owned(),
        command: "git remote".to_owned(),
        condition: Some(|simple, _, _| simple.args.len() == 1),
        outcome: Outcome::allow("Read-only `git remote`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn git_safe_subcommands() {
        for sub in [
            "blame",
            "check-ignore",
            "describe",
            "diff",
            "fetch",
            "grep",
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

    /// Regex pattern with flags.
    #[test]
    fn git_grep() {
        let outcome = evaluate_expect_outcome("git grep -E 'foo|bar'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_log_args() {
        let outcome = evaluate_expect_outcome("git log --oneline -5");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_patch_id() {
        let outcome = evaluate_expect_outcome("git patch-id --stable");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_log_pipe_patch_id() {
        let outcome = evaluate_expect_outcome("git log --format='%H' main | git patch-id --stable");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_diff_head_1() {
        let outcome = evaluate_expect_outcome("git diff HEAD~1");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_show() {
        let outcome = evaluate_expect_outcome("git remote show origin");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_get_url() {
        let outcome = evaluate_expect_outcome("git remote get-url origin");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_commit() {
        let reason = evaluate_expect_skip("git commit -m 'test'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_push() {
        let reason = evaluate_expect_skip("git push origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_add() {
        let reason = evaluate_expect_skip("git add -A");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_rebase() {
        let reason = evaluate_expect_skip("git rebase main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_hard() {
        let outcome = evaluate_expect_outcome("git reset --hard HEAD~1");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard() {
        let outcome = evaluate_expect_outcome("git checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop() {
        let outcome = evaluate_expect_outcome("git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_remote_add() {
        let reason = evaluate_expect_skip("git remote add upstream https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn ls() {
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo() {
        let reason = evaluate_expect_skip("cargo publish");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cat() {
        let outcome = evaluate_expect_outcome("cat file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_no_pager_log() {
        let reason = evaluate_expect_skip("git --no-pager log");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_no_pager_diff() {
        let reason = evaluate_expect_skip("git --no-pager diff HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_c_config() {
        let reason = evaluate_expect_skip("git -c core.pager= status");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn ls_git_status() {
        let outcome = evaluate_expect_outcome("ls && git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_add_commit() {
        let reason = evaluate_expect_skip("git add file.txt && git commit -m 'test'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_log_or_diff() {
        let outcome = evaluate_expect_outcome("git log || git diff");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_status_semicolon_log() {
        let outcome = evaluate_expect_outcome("git status ; git log");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_status_pipe() {
        let outcome = evaluate_expect_outcome("git status | head -5");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_git() {
        let outcome = evaluate_expect_outcome("echo git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_bare() {
        let outcome = evaluate_expect_outcome("git branch");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_a() {
        let outcome = evaluate_expect_outcome("git branch -a");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_list() {
        let outcome = evaluate_expect_outcome("git branch --list");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_r() {
        let outcome = evaluate_expect_outcome("git branch -r");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_v() {
        let outcome = evaluate_expect_outcome("git branch -v");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_vv() {
        let outcome = evaluate_expect_outcome("git branch -vv");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_contains() {
        let outcome = evaluate_expect_outcome("git branch --contains");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_merged() {
        let outcome = evaluate_expect_outcome("git branch --merged");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_no_merged() {
        let outcome = evaluate_expect_outcome("git branch --no-merged");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_sort() {
        let reason = evaluate_expect_skip("git branch --sort=committerdate");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_format() {
        let reason = evaluate_expect_skip("git branch --format='%(refname:short)'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_show_current() {
        let outcome = evaluate_expect_outcome("git branch --show-current");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_points_at() {
        let outcome = evaluate_expect_outcome("git branch --points-at HEAD");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_combined() {
        let outcome = evaluate_expect_outcome("git branch -a -v --sort=committerdate");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_d() {
        let reason = evaluate_expect_skip("git branch -d old-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_cap_d() {
        let reason = evaluate_expect_skip("git branch -D old-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_m() {
        let reason = evaluate_expect_skip("git branch -m old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_cap_m() {
        let reason = evaluate_expect_skip("git branch -M old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_c() {
        let reason = evaluate_expect_skip("git branch -c old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_delete() {
        let reason = evaluate_expect_skip("git branch --delete old-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_move() {
        let reason = evaluate_expect_skip("git branch --move old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_copy() {
        let reason = evaluate_expect_skip("git branch --copy old new");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_set_upstream() {
        let reason = evaluate_expect_skip("git branch --set-upstream-to=origin/main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_unset_upstream() {
        let reason = evaluate_expect_skip("git branch --unset-upstream");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_bare() {
        let outcome = evaluate_expect_outcome("git tag");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_l() {
        let outcome = evaluate_expect_outcome("git tag -l");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_list() {
        let outcome = evaluate_expect_outcome("git tag --list");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_n() {
        let reason = evaluate_expect_skip("git tag -n");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_n5() {
        let reason = evaluate_expect_skip("git tag -n5");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_contains() {
        let outcome = evaluate_expect_outcome("git tag --contains");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_contains_commit() {
        let outcome = evaluate_expect_outcome("git tag --contains f4ce32b");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_merged() {
        let outcome = evaluate_expect_outcome("git tag --merged");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_merged_commit() {
        let outcome = evaluate_expect_outcome("git tag --merged main");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_no_merged() {
        let outcome = evaluate_expect_outcome("git tag --no-merged");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_no_merged_commit() {
        let outcome = evaluate_expect_outcome("git tag --no-merged main");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_list_pattern() {
        let outcome = evaluate_expect_outcome("git tag -l 'v1.*'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_sort() {
        let outcome = evaluate_expect_outcome("git tag --sort=version:refname");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_sort_negative() {
        let outcome = evaluate_expect_outcome("git tag --sort=-creatordate");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Space-separated value; `--sort` alone triggers the `with_any` match.
    #[test]
    fn git_tag_sort_space_separated() {
        let outcome = evaluate_expect_outcome("git tag --sort -creatordate");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_verify() {
        let outcome = evaluate_expect_outcome("git tag -v v1.0");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_verify_long() {
        let outcome = evaluate_expect_outcome("git tag --verify v1.0");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_d() {
        let reason = evaluate_expect_skip("git tag -d v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_a() {
        let reason = evaluate_expect_skip("git tag -a v1.0 -m 'release'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_s() {
        let reason = evaluate_expect_skip("git tag -s v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_f() {
        let reason = evaluate_expect_skip("git tag -f v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_m() {
        let reason = evaluate_expect_skip("git tag -m 'release'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_positional() {
        let reason = evaluate_expect_skip("git tag v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_delete() {
        let reason = evaluate_expect_skip("git tag --delete v1.0");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_bare() {
        let outcome = evaluate_expect_outcome("git remote");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_v() {
        let outcome = evaluate_expect_outcome("git remote -v");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_verbose() {
        let outcome = evaluate_expect_outcome("git remote --verbose");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_show_origin() {
        let outcome = evaluate_expect_outcome("git remote show origin");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_get_url_origin() {
        let outcome = evaluate_expect_outcome("git remote get-url origin");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add_upstream() {
        let reason = evaluate_expect_skip("git remote add upstream https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_remove() {
        let reason = evaluate_expect_skip("git remote remove upstream");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_rename() {
        let reason = evaluate_expect_skip("git remote rename origin upstream");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_set_url() {
        let reason = evaluate_expect_skip("git remote set-url origin https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_set_head() {
        let reason = evaluate_expect_skip("git remote set-head origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_set_branches() {
        let reason = evaluate_expect_skip("git remote set-branches origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_prune() {
        let reason = evaluate_expect_skip("git remote prune origin");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_blame() {
        let outcome = evaluate_expect_outcome("git blame file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_blame_line_range() {
        let outcome = evaluate_expect_outcome("git blame file.txt -L 15,45");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_worktree_list() {
        let outcome = evaluate_expect_outcome("git worktree list");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_worktree_list_porcelain() {
        let outcome = evaluate_expect_outcome("git worktree list --porcelain");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_worktree_add() {
        let reason = evaluate_expect_skip("git worktree add ../foo main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_remove() {
        let reason = evaluate_expect_skip("git worktree remove ../foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_prune() {
        let reason = evaluate_expect_skip("git worktree prune");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_move() {
        let reason = evaluate_expect_skip("git worktree move ../foo ../bar");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_lock() {
        let reason = evaluate_expect_skip("git worktree lock ../foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_unlock() {
        let reason = evaluate_expect_skip("git worktree unlock ../foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_repair() {
        let reason = evaluate_expect_skip("git worktree repair");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_bare() {
        let reason = evaluate_expect_skip("git worktree");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_list() {
        let outcome = evaluate_expect_outcome("git config list");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get() {
        let outcome = evaluate_expect_outcome("git config get core.hooksPath");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_list_long() {
        let outcome = evaluate_expect_outcome("git config --list");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_l() {
        let outcome = evaluate_expect_outcome("git config -l");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_flag() {
        let outcome = evaluate_expect_outcome("git config --get core.hooksPath");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_all() {
        let outcome = evaluate_expect_outcome("git config --get-all remote.origin.fetch");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_regexp() {
        let outcome = evaluate_expect_outcome("git config --get-regexp '^remote\\.'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_urlmatch() {
        let outcome = evaluate_expect_outcome("git config --get-urlmatch http https://example.com");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_colorbool() {
        let outcome = evaluate_expect_outcome("git config --get-colorbool color.diff");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_set() {
        let reason = evaluate_expect_skip("git config set user.name foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_unset() {
        let reason = evaluate_expect_skip("git config unset user.name");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_set_flag() {
        let reason = evaluate_expect_skip("git config --set user.name foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_add() {
        let reason = evaluate_expect_skip("git config --add remote.origin.fetch refs/foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_unset_flag() {
        let reason = evaluate_expect_skip("git config --unset user.name");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_replace_all() {
        let reason = evaluate_expect_skip("git config --replace-all user.name foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_edit() {
        let reason = evaluate_expect_skip("git config edit");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_remove_section() {
        let reason = evaluate_expect_skip("git config remove-section user");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_rename_section() {
        let reason = evaluate_expect_skip("git config rename-section foo bar");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Bare `git config <key> <value>` writes the value; must not be allowed.
    #[test]
    fn git_config_bare_set() {
        let reason = evaluate_expect_skip("git config user.name foo");
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
