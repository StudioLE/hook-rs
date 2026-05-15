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
        condition: Some(|ctx| ctx.simple.args.len() == 1),
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
            with_any: Some(vec![ArgMatcher::new(flag)]),
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
        condition: Some(|ctx| ctx.simple.args.len() == 1),
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
            ArgMatcher::new("-l"),
            ArgMatcher::new("--list"),
            ArgMatcher::new("-v"),
            ArgMatcher::new("--verify"),
            ArgMatcher::new("--contains"),
            ArgMatcher::new("--merged"),
            ArgMatcher::new("--no-merged"),
            ArgMatcher::new("--sort"),
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
            ArgMatcher::new("-l"),
            ArgMatcher::new("--list"),
            ArgMatcher::new("--get"),
            ArgMatcher::new("--get-all"),
            ArgMatcher::new("--get-regexp"),
            ArgMatcher::new("--get-urlmatch"),
            ArgMatcher::new("--get-color"),
            ArgMatcher::new("--get-colorbool"),
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
        condition: Some(|ctx| ctx.simple.args.len() == 1),
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
            let result = eval_rules(git_allow_rules(), &format!("git {sub}"));
            let outcome = expect_outcome(result);
            assert_eq!(outcome.decision, Decision::Allow, "git {sub}");
        }
    }

    /// Regex pattern with flags.
    #[test]
    fn git_grep() {
        let result = eval_rules(git_allow_rules(), "git grep -E 'foo|bar'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_log_args() {
        let result = eval_rules(git_allow_rules(), "git log --oneline -5");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_patch_id() {
        let result = eval_rules(git_allow_rules(), "git patch-id --stable");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_log_pipe_patch_id() {
        let result = eval_rules(
            git_allow_rules(),
            "git log --format='%H' main | git patch-id --stable",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_diff_head_1() {
        let result = eval_rules(git_allow_rules(), "git diff HEAD~1");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_show() {
        let result = eval_rules(git_allow_rules(), "git remote show origin");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_get_url() {
        let result = eval_rules(git_allow_rules(), "git remote get-url origin");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_commit() {
        let result = eval_rules(git_allow_rules(), "git commit -m 'test'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_push() {
        let result = eval_rules(git_allow_rules(), "git push origin main");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_add() {
        let result = eval_rules(git_allow_rules(), "git add -A");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_rebase() {
        let result = eval_rules(git_allow_rules(), "git rebase main");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_hard() {
        let result = eval_rules(git_allow_rules(), "git reset --hard HEAD~1");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_discard() {
        let result = eval_rules(git_allow_rules(), "git checkout -- file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_pop() {
        let result = eval_rules(git_allow_rules(), "git stash pop");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_add() {
        let result = eval_rules(
            git_allow_rules(),
            "git remote add upstream https://example.com",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn ls() {
        let result = eval_rules(git_allow_rules(), "ls -la");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo() {
        let result = eval_rules(git_allow_rules(), "cargo publish");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cat() {
        let result = eval_rules(git_allow_rules(), "cat file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_no_pager_log() {
        let result = eval_rules(git_allow_rules(), "git --no-pager log");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_no_pager_diff() {
        let result = eval_rules(git_allow_rules(), "git --no-pager diff HEAD~1");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_c_config() {
        let result = eval_rules(git_allow_rules(), "git -c core.pager= status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn ls_git_status() {
        let result = eval_rules(git_allow_rules(), "ls && git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn git_add_commit() {
        let result = eval_rules(
            git_allow_rules(),
            "git add file.txt && git commit -m 'test'",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_log_or_diff() {
        let result = eval_rules(git_allow_rules(), "git log || git diff");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_status_semicolon_log() {
        let result = eval_rules(git_allow_rules(), "git status ; git log");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_status_pipe() {
        let result = eval_rules(git_allow_rules(), "git status | head -5");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn echo_git() {
        let result = eval_rules(git_allow_rules(), "echo git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_bare() {
        let result = eval_rules(git_allow_rules(), "git branch");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_a() {
        let result = eval_rules(git_allow_rules(), "git branch -a");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_list() {
        let result = eval_rules(git_allow_rules(), "git branch --list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_r() {
        let result = eval_rules(git_allow_rules(), "git branch -r");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_v() {
        let result = eval_rules(git_allow_rules(), "git branch -v");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_vv() {
        let result = eval_rules(git_allow_rules(), "git branch -vv");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_contains() {
        let result = eval_rules(git_allow_rules(), "git branch --contains");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_merged() {
        let result = eval_rules(git_allow_rules(), "git branch --merged");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_no_merged() {
        let result = eval_rules(git_allow_rules(), "git branch --no-merged");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_sort() {
        let result = eval_rules(git_allow_rules(), "git branch --sort=committerdate");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_format() {
        let result = eval_rules(git_allow_rules(), "git branch --format='%(refname:short)'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_show_current() {
        let result = eval_rules(git_allow_rules(), "git branch --show-current");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_points_at() {
        let result = eval_rules(git_allow_rules(), "git branch --points-at HEAD");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_combined() {
        let result = eval_rules(git_allow_rules(), "git branch -a -v --sort=committerdate");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_d() {
        let result = eval_rules(git_allow_rules(), "git branch -d old-branch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_cap_d() {
        let result = eval_rules(git_allow_rules(), "git branch -D old-branch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_m() {
        let result = eval_rules(git_allow_rules(), "git branch -m old new");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_cap_m() {
        let result = eval_rules(git_allow_rules(), "git branch -M old new");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_c() {
        let result = eval_rules(git_allow_rules(), "git branch -c old new");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_delete() {
        let result = eval_rules(git_allow_rules(), "git branch --delete old-branch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_move() {
        let result = eval_rules(git_allow_rules(), "git branch --move old new");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_copy() {
        let result = eval_rules(git_allow_rules(), "git branch --copy old new");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_set_upstream() {
        let result = eval_rules(
            git_allow_rules(),
            "git branch --set-upstream-to=origin/main",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_branch_unset_upstream() {
        let result = eval_rules(git_allow_rules(), "git branch --unset-upstream");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_bare() {
        let result = eval_rules(git_allow_rules(), "git tag");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_l() {
        let result = eval_rules(git_allow_rules(), "git tag -l");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_list() {
        let result = eval_rules(git_allow_rules(), "git tag --list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_n() {
        let result = eval_rules(git_allow_rules(), "git tag -n");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_n5() {
        let result = eval_rules(git_allow_rules(), "git tag -n5");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_contains() {
        let result = eval_rules(git_allow_rules(), "git tag --contains");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_contains_commit() {
        let result = eval_rules(git_allow_rules(), "git tag --contains f4ce32b");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_merged() {
        let result = eval_rules(git_allow_rules(), "git tag --merged");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_merged_commit() {
        let result = eval_rules(git_allow_rules(), "git tag --merged main");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_no_merged() {
        let result = eval_rules(git_allow_rules(), "git tag --no-merged");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_no_merged_commit() {
        let result = eval_rules(git_allow_rules(), "git tag --no-merged main");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_list_pattern() {
        let result = eval_rules(git_allow_rules(), "git tag -l 'v1.*'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_sort() {
        let result = eval_rules(git_allow_rules(), "git tag --sort=version:refname");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_sort_negative() {
        let result = eval_rules(git_allow_rules(), "git tag --sort=-creatordate");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Space-separated value; `--sort` alone triggers the `with_any` match.
    #[test]
    fn git_tag_sort_space_separated() {
        let result = eval_rules(git_allow_rules(), "git tag --sort -creatordate");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_verify() {
        let result = eval_rules(git_allow_rules(), "git tag -v v1.0");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_verify_long() {
        let result = eval_rules(git_allow_rules(), "git tag --verify v1.0");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_d() {
        let result = eval_rules(git_allow_rules(), "git tag -d v1.0");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_a() {
        let result = eval_rules(git_allow_rules(), "git tag -a v1.0 -m 'release'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_s() {
        let result = eval_rules(git_allow_rules(), "git tag -s v1.0");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_f() {
        let result = eval_rules(git_allow_rules(), "git tag -f v1.0");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_m() {
        let result = eval_rules(git_allow_rules(), "git tag -m 'release'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_positional() {
        let result = eval_rules(git_allow_rules(), "git tag v1.0");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_tag_delete() {
        let result = eval_rules(git_allow_rules(), "git tag --delete v1.0");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_bare() {
        let result = eval_rules(git_allow_rules(), "git remote");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_v() {
        let result = eval_rules(git_allow_rules(), "git remote -v");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_verbose() {
        let result = eval_rules(git_allow_rules(), "git remote --verbose");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_show_origin() {
        let result = eval_rules(git_allow_rules(), "git remote show origin");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_get_url_origin() {
        let result = eval_rules(git_allow_rules(), "git remote get-url origin");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add_upstream() {
        let result = eval_rules(
            git_allow_rules(),
            "git remote add upstream https://example.com",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_remove() {
        let result = eval_rules(git_allow_rules(), "git remote remove upstream");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_rename() {
        let result = eval_rules(git_allow_rules(), "git remote rename origin upstream");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_set_url() {
        let result = eval_rules(
            git_allow_rules(),
            "git remote set-url origin https://example.com",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_set_head() {
        let result = eval_rules(git_allow_rules(), "git remote set-head origin main");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_set_branches() {
        let result = eval_rules(git_allow_rules(), "git remote set-branches origin main");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_remote_prune() {
        let result = eval_rules(git_allow_rules(), "git remote prune origin");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_blame() {
        let result = eval_rules(git_allow_rules(), "git blame file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_blame_line_range() {
        let result = eval_rules(git_allow_rules(), "git blame file.txt -L 15,45");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_worktree_list() {
        let result = eval_rules(git_allow_rules(), "git worktree list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_worktree_list_porcelain() {
        let result = eval_rules(git_allow_rules(), "git worktree list --porcelain");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_worktree_remove() {
        let result = eval_rules(git_allow_rules(), "git worktree remove ../foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_prune() {
        let result = eval_rules(git_allow_rules(), "git worktree prune");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_move() {
        let result = eval_rules(git_allow_rules(), "git worktree move ../foo ../bar");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_lock() {
        let result = eval_rules(git_allow_rules(), "git worktree lock ../foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_unlock() {
        let result = eval_rules(git_allow_rules(), "git worktree unlock ../foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_repair() {
        let result = eval_rules(git_allow_rules(), "git worktree repair");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_worktree_bare() {
        let result = eval_rules(git_allow_rules(), "git worktree");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_list() {
        let result = eval_rules(git_allow_rules(), "git config list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get() {
        let result = eval_rules(git_allow_rules(), "git config get core.hooksPath");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_list_long() {
        let result = eval_rules(git_allow_rules(), "git config --list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_l() {
        let result = eval_rules(git_allow_rules(), "git config -l");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_flag() {
        let result = eval_rules(git_allow_rules(), "git config --get core.hooksPath");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_all() {
        let result = eval_rules(
            git_allow_rules(),
            "git config --get-all remote.origin.fetch",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_regexp() {
        let result = eval_rules(git_allow_rules(), "git config --get-regexp '^remote\\.'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_urlmatch() {
        let result = eval_rules(
            git_allow_rules(),
            "git config --get-urlmatch http https://example.com",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_get_colorbool() {
        let result = eval_rules(git_allow_rules(), "git config --get-colorbool color.diff");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_config_set() {
        let result = eval_rules(git_allow_rules(), "git config set user.name foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_unset() {
        let result = eval_rules(git_allow_rules(), "git config unset user.name");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_set_flag() {
        let result = eval_rules(git_allow_rules(), "git config --set user.name foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_add() {
        let result = eval_rules(
            git_allow_rules(),
            "git config --add remote.origin.fetch refs/foo",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_unset_flag() {
        let result = eval_rules(git_allow_rules(), "git config --unset user.name");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_replace_all() {
        let result = eval_rules(git_allow_rules(), "git config --replace-all user.name foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_edit() {
        let result = eval_rules(git_allow_rules(), "git config edit");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_remove_section() {
        let result = eval_rules(git_allow_rules(), "git config remove-section user");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_config_rename_section() {
        let result = eval_rules(git_allow_rules(), "git config rename-section foo bar");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Bare `git config <key> <value>` writes the value; must not be allowed.
    #[test]
    fn git_config_bare_set() {
        let result = eval_rules(git_allow_rules(), "git config user.name foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
