//! Allow read-only git subcommands and classify repository paths by trust level.

use crate::prelude::*;

/// Allow read-only git subcommands and classify paths by trust level.
#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    if !parsed.is_standalone() {
        return None;
    }
    let cmd = parsed.all_commands().next()?;
    let ga = parse_git_args(cmd)?;
    if ga.args.is_empty() {
        return None;
    }
    let subcommand = safe_subcommand(ga.args);
    let path = ga.path.as_deref().unwrap_or("");
    let path_class = classify_path(path);
    if let Some(sub) = subcommand {
        match path_class {
            PathClass::None | PathClass::Trusted => {
                Some(CheckResult::allow(format!("Safe git subcommand: {sub}")))
            }
            PathClass::Forked => Some(CheckResult::ask(format!(
                "git {sub} in forked repo: {path}"
            ))),
            PathClass::Unknown => Some(CheckResult::ask(format!(
                "git {sub} in unknown path: {path}"
            ))),
        }
    } else if path_class == PathClass::None {
        None
    } else {
        let first_arg = ga.args.first()?;
        if matches!(first_arg.as_str(), "stash" | "reset" | "checkout" | "clean") {
            return None;
        }
        Some(CheckResult::ask(format!("git {first_arg} -C {path}")))
    }
}

fn safe_subcommand(args: &[String]) -> Option<&str> {
    let first = args.first()?.as_str();
    let rest = args.get(1..).unwrap_or_default();
    match first {
        "check-ignore" | "describe" | "diff" | "fetch" | "log" | "ls-tree" | "merge-base"
        | "mv" | "rev-parse" | "rm" | "show" | "status" => Some(first),
        "remote" => check_remote_subcommand(rest),
        "branch" => check_branch_flags(rest),
        "tag" => check_tag_flags(rest),
        _ => None,
    }
}

fn check_remote_subcommand(rest: &[String]) -> Option<&'static str> {
    match rest.first().map(String::as_str) {
        None | Some("-v" | "--verbose") => Some("remote"),
        Some("show") => Some("remote show"),
        Some("get-url") => Some("remote get-url"),
        Some("add" | "remove" | "rename" | "set-url" | "set-head" | "set-branches" | "prune") => {
            None
        }
        Some(_) => None,
    }
}

fn check_branch_flags(rest: &[String]) -> Option<&'static str> {
    if rest.is_empty() {
        return Some("branch");
    }
    for arg in rest {
        if arg.starts_with('-') {
            if is_branch_read_flag(arg) {
                continue;
            }
            if is_branch_write_flag(arg) {
                return None;
            }
            return None;
        }
    }
    Some("branch")
}

fn is_branch_read_flag(flag: &str) -> bool {
    matches!(
        flag,
        "-a" | "--all"
            | "-l"
            | "--list"
            | "-r"
            | "--remotes"
            | "-v"
            | "--verbose"
            | "-vv"
            | "--contains"
            | "--merged"
            | "--no-merged"
    ) || flag.starts_with("--sort=")
        || flag.starts_with("--format=")
        || flag == "--points-at"
}

fn is_branch_write_flag(flag: &str) -> bool {
    matches!(
        flag,
        "-d" | "-D"
            | "-m"
            | "-M"
            | "-c"
            | "-C"
            | "--delete"
            | "--move"
            | "--copy"
            | "--set-upstream-to"
            | "--unset-upstream"
    ) || flag.starts_with("--set-upstream-to=")
}

fn check_tag_flags(rest: &[String]) -> Option<&'static str> {
    if rest.is_empty() {
        return Some("tag");
    }
    for arg in rest {
        if arg.starts_with('-') {
            if is_tag_read_flag(arg) {
                continue;
            }
            if is_tag_write_flag(arg) {
                return None;
            }
            return None;
        }
        // Positional arg (tag name for creation) — not read-only
        return None;
    }
    Some("tag")
}

fn is_tag_read_flag(flag: &str) -> bool {
    matches!(
        flag,
        "-l" | "--list" | "-n" | "--contains" | "--merged" | "--no-merged" | "-v" | "--verify"
    ) || flag.starts_with("--sort=")
        || flag.starts_with("--format=")
        || flag.starts_with("-n")
}

fn is_tag_write_flag(flag: &str) -> bool {
    matches!(flag, "-d" | "-a" | "-s" | "-f" | "-m" | "--delete")
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check(command: &str) -> Option<CheckResult> {
        let parsed = parse(command)?;
        super::check(&parsed)
    }

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
            let result = check(&format!("git {sub}"));
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
        assert_yaml_snapshot!(check("git log --oneline -5"));
    }

    #[test]
    fn diff_head_1() {
        assert_yaml_snapshot!(check("git diff HEAD~1"));
    }

    #[test]
    fn remote_show() {
        assert_yaml_snapshot!(check("git remote show origin"));
    }

    #[test]
    fn remote_get_url() {
        assert_yaml_snapshot!(check("git remote get-url origin"));
    }

    #[test]
    fn trusted_path_status() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura status"));
    }

    #[test]
    fn trusted_path_log() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/alnwick log --oneline"));
    }

    #[test]
    fn trusted_subdir_diff() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Infrastructure/homelab diff"));
    }

    #[test]
    fn double_quoted_trusted_path() {
        assert_yaml_snapshot!(check("git -C \"/var/mnt/e/Repos/Rust/caesura\" status"));
    }

    #[test]
    fn single_quoted_trusted_path() {
        assert_yaml_snapshot!(check("git -C '/var/mnt/e/Repos/Rust/caesura' status"));
    }

    #[test]
    fn trailing_slash_trusted_path() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura/ status"));
    }

    #[test]
    fn forked_status() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Forked/some-repo status"));
    }

    #[test]
    fn forked_log() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Forked/some-repo log"));
    }

    #[test]
    fn unknown_status() {
        assert_yaml_snapshot!(check("git -C /tmp/sketchy-repo status"));
    }

    #[test]
    fn unknown_diff() {
        assert_yaml_snapshot!(check("git -C /home/other/repo diff"));
    }

    #[test]
    fn unsafe_commit_passthrough() {
        assert_eq!(check("git commit -m 'test'"), None);
    }

    #[test]
    fn unsafe_push_passthrough() {
        assert_eq!(check("git push origin main"), None);
    }

    #[test]
    fn unsafe_add_passthrough() {
        assert_eq!(check("git add -A"), None);
    }

    #[test]
    fn unsafe_rebase_passthrough() {
        assert_eq!(check("git rebase main"), None);
    }

    #[test]
    fn unsafe_reset_passthrough() {
        assert_eq!(check("git reset --hard HEAD~1"), None);
    }

    #[test]
    fn unsafe_checkout_passthrough() {
        assert_eq!(check("git checkout -- file.txt"), None);
    }

    #[test]
    fn unsafe_stash_passthrough() {
        assert_eq!(check("git stash pop"), None);
    }

    #[test]
    fn unsafe_remote_add_passthrough() {
        assert_eq!(check("git remote add upstream https://example.com"), None);
    }

    #[test]
    fn unsafe_with_c_path_commit() {
        assert_yaml_snapshot!(check(
            "git -C /var/mnt/e/Repos/Rust/caesura commit -m 'test'"
        ));
    }

    #[test]
    fn unsafe_with_c_path_push() {
        assert_yaml_snapshot!(check(
            "git -C /var/mnt/e/Repos/Rust/caesura push origin main"
        ));
    }

    #[test]
    fn unsafe_with_c_path_add() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura add -A"));
    }

    #[test]
    fn unsafe_unknown_commit() {
        assert_yaml_snapshot!(check("git -C /tmp/sketchy commit -m 'evil'"));
    }

    #[test]
    fn non_git_ls_passthrough() {
        assert_eq!(check("ls -la"), None);
    }

    #[test]
    fn non_git_cargo_passthrough() {
        assert_eq!(check("cargo build"), None);
    }

    #[test]
    fn non_git_cat_passthrough() {
        assert_eq!(check("cat file.txt"), None);
    }

    #[test]
    fn options_before_subcommand_passthrough() {
        assert_eq!(check("git --no-pager log"), None);
    }

    #[test]
    fn options_no_pager_diff_passthrough() {
        assert_eq!(check("git --no-pager diff HEAD~1"), None);
    }

    #[test]
    fn options_c_config_passthrough() {
        assert_eq!(check("git -c core.pager= status"), None);
    }

    #[test]
    fn chained_ls_git_passthrough() {
        assert_eq!(check("ls && git status"), None);
    }

    #[test]
    fn chained_git_git_passthrough() {
        assert_eq!(check("git add file.txt && git commit -m 'test'"), None);
    }

    #[test]
    fn chained_git_c_push_passthrough() {
        assert_eq!(check("git status && git -C /tmp/evil push"), None);
    }

    #[test]
    fn chained_or_passthrough() {
        assert_eq!(check("git log || git diff"), None);
    }

    #[test]
    fn chained_semicolon_passthrough() {
        assert_eq!(check("git status ; git log"), None);
    }

    #[test]
    fn chained_pipe_passthrough() {
        assert_eq!(check("git status | head -5"), None);
    }

    #[test]
    fn grep_c_passthrough() {
        assert_eq!(check("grep -C 3 pattern file"), None);
    }

    #[test]
    fn echo_git_passthrough() {
        assert_eq!(check("echo git status"), None);
    }

    #[test]
    fn echo_git_c_quoted_passthrough() {
        assert_eq!(check("echo 'git -C /path status'"), None);
    }

    // Branch flag tests
    #[test]
    fn branch_bare() {
        assert_yaml_snapshot!(check("git branch"));
    }

    #[test]
    fn branch_all() {
        assert_yaml_snapshot!(check("git branch -a"));
    }

    #[test]
    fn branch_list() {
        assert_yaml_snapshot!(check("git branch --list"));
    }

    #[test]
    fn branch_remotes() {
        assert_yaml_snapshot!(check("git branch -r"));
    }

    #[test]
    fn branch_verbose() {
        assert_yaml_snapshot!(check("git branch -v"));
    }

    #[test]
    fn branch_double_verbose() {
        assert_yaml_snapshot!(check("git branch -vv"));
    }

    #[test]
    fn branch_contains() {
        assert_yaml_snapshot!(check("git branch --contains"));
    }

    #[test]
    fn branch_merged() {
        assert_yaml_snapshot!(check("git branch --merged"));
    }

    #[test]
    fn branch_no_merged() {
        assert_yaml_snapshot!(check("git branch --no-merged"));
    }

    #[test]
    fn branch_sort() {
        assert_yaml_snapshot!(check("git branch --sort=committerdate"));
    }

    #[test]
    fn branch_format() {
        assert_yaml_snapshot!(check("git branch --format='%(refname:short)'"));
    }

    #[test]
    fn branch_points_at() {
        assert_yaml_snapshot!(check("git branch --points-at HEAD"));
    }

    #[test]
    fn branch_combined_read_flags() {
        assert_yaml_snapshot!(check("git branch -a -v --sort=committerdate"));
    }

    #[test]
    fn branch_delete_passthrough() {
        assert_eq!(check("git branch -d old-branch"), None);
    }

    #[test]
    fn branch_force_delete_passthrough() {
        assert_eq!(check("git branch -D old-branch"), None);
    }

    #[test]
    fn branch_move_passthrough() {
        assert_eq!(check("git branch -m old new"), None);
    }

    #[test]
    fn branch_force_move_passthrough() {
        assert_eq!(check("git branch -M old new"), None);
    }

    #[test]
    fn branch_copy_passthrough() {
        assert_eq!(check("git branch -c old new"), None);
    }

    #[test]
    fn branch_long_delete_passthrough() {
        assert_eq!(check("git branch --delete old-branch"), None);
    }

    #[test]
    fn branch_long_move_passthrough() {
        assert_eq!(check("git branch --move old new"), None);
    }

    #[test]
    fn branch_long_copy_passthrough() {
        assert_eq!(check("git branch --copy old new"), None);
    }

    #[test]
    fn branch_set_upstream_passthrough() {
        assert_eq!(check("git branch --set-upstream-to=origin/main"), None);
    }

    #[test]
    fn branch_unset_upstream_passthrough() {
        assert_eq!(check("git branch --unset-upstream"), None);
    }

    // Tag flag tests
    #[test]
    fn tag_bare() {
        assert_yaml_snapshot!(check("git tag"));
    }

    #[test]
    fn tag_list() {
        assert_yaml_snapshot!(check("git tag -l"));
    }

    #[test]
    fn tag_list_long() {
        assert_yaml_snapshot!(check("git tag --list"));
    }

    #[test]
    fn tag_n() {
        assert_yaml_snapshot!(check("git tag -n"));
    }

    #[test]
    fn tag_n5() {
        assert_yaml_snapshot!(check("git tag -n5"));
    }

    #[test]
    fn tag_contains() {
        assert_yaml_snapshot!(check("git tag --contains"));
    }

    #[test]
    fn tag_merged() {
        assert_yaml_snapshot!(check("git tag --merged"));
    }

    #[test]
    fn tag_no_merged() {
        assert_yaml_snapshot!(check("git tag --no-merged"));
    }

    #[test]
    fn tag_sort() {
        assert_yaml_snapshot!(check("git tag --sort=version:refname"));
    }

    #[test]
    fn tag_verify() {
        assert_yaml_snapshot!(check("git tag -v v1.0"));
    }

    #[test]
    fn tag_verify_long() {
        assert_yaml_snapshot!(check("git tag --verify v1.0"));
    }

    #[test]
    fn tag_delete_passthrough() {
        assert_eq!(check("git tag -d v1.0"), None);
    }

    #[test]
    fn tag_annotated_passthrough() {
        assert_eq!(check("git tag -a v1.0 -m 'release'"), None);
    }

    #[test]
    fn tag_signed_passthrough() {
        assert_eq!(check("git tag -s v1.0"), None);
    }

    #[test]
    fn tag_force_passthrough() {
        assert_eq!(check("git tag -f v1.0"), None);
    }

    #[test]
    fn tag_message_passthrough() {
        assert_eq!(check("git tag -m 'release'"), None);
    }

    #[test]
    fn tag_positional_create_passthrough() {
        assert_eq!(check("git tag v1.0"), None);
    }

    #[test]
    fn tag_long_delete_passthrough() {
        assert_eq!(check("git tag --delete v1.0"), None);
    }

    // Remote flag tests
    #[test]
    fn remote_bare() {
        assert_yaml_snapshot!(check("git remote"));
    }

    #[test]
    fn remote_verbose() {
        assert_yaml_snapshot!(check("git remote -v"));
    }

    #[test]
    fn remote_verbose_long() {
        assert_yaml_snapshot!(check("git remote --verbose"));
    }

    #[test]
    fn remote_show_origin() {
        assert_yaml_snapshot!(check("git remote show origin"));
    }

    #[test]
    fn remote_get_url_origin() {
        assert_yaml_snapshot!(check("git remote get-url origin"));
    }

    #[test]
    fn remote_add_passthrough() {
        assert_eq!(check("git remote add upstream https://example.com"), None);
    }

    #[test]
    fn remote_remove_passthrough() {
        assert_eq!(check("git remote remove upstream"), None);
    }

    #[test]
    fn remote_rename_passthrough() {
        assert_eq!(check("git remote rename origin upstream"), None);
    }

    #[test]
    fn remote_set_url_passthrough() {
        assert_eq!(check("git remote set-url origin https://example.com"), None);
    }

    #[test]
    fn remote_set_head_passthrough() {
        assert_eq!(check("git remote set-head origin main"), None);
    }

    #[test]
    fn remote_set_branches_passthrough() {
        assert_eq!(check("git remote set-branches origin main"), None);
    }

    #[test]
    fn remote_prune_passthrough() {
        assert_eq!(check("git remote prune origin"), None);
    }

    // Branch/tag/remote with -C path
    #[test]
    fn branch_trusted_path() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura branch -a"));
    }

    #[test]
    fn tag_trusted_path() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura tag -l"));
    }

    #[test]
    fn remote_trusted_path() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura remote -v"));
    }

    #[test]
    fn branch_forked_path() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Forked/repo branch"));
    }

    #[test]
    fn branch_delete_with_path() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura branch -d old"));
    }

    #[test]
    fn stash_with_c_path_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura stash pop"),
            None
        );
    }

    #[test]
    fn reset_with_c_path_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura reset --hard"),
            None
        );
    }

    #[test]
    fn checkout_with_c_path_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura checkout -- file.txt"),
            None
        );
    }

    #[test]
    fn clean_with_c_path_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura clean -fd"),
            None
        );
    }
}
