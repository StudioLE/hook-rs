use crate::prelude::*;

pub fn git_rules() -> Vec<SimpleRule> {
    vec![
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_reset_hard),
            outcome: Outcome::deny("git reset --hard discards uncommitted changes"),
            ..Default::default()
        },
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_stash_pop),
            outcome: Outcome::deny("git stash pop can cause merge conflicts and lose stash"),
            ..Default::default()
        },
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_stash_drop),
            outcome: Outcome::deny("git stash drop permanently deletes a stash entry"),
            ..Default::default()
        },
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_stash_clear),
            outcome: Outcome::deny("git stash clear permanently deletes all stash entries"),
            ..Default::default()
        },
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_clean_d),
            outcome: Outcome::deny(
                "git clean with -d is blocked. Use 'git clean -f <file>' for specific files \
                 (or -fx if gitignored) or 'git rm -r <dir>' for tracked directories.",
            ),
            ..Default::default()
        },
    ]
}

fn git_subcommand_is(cmd: &SimpleContext, sub: &str, opt: &str) -> bool {
    let Some(ga) = parse_git_args(cmd) else {
        return false;
    };
    ga.args.first().is_some_and(|a| a == sub) && ga.args.get(1).is_some_and(|a| a == opt)
}

fn is_reset_hard(cmd: &SimpleContext) -> bool {
    let Some(ga) = parse_git_args(cmd) else {
        return false;
    };
    ga.args.first().is_some_and(|a| a == "reset") && ga.args.iter().any(|a| a == "--hard")
}

fn is_stash_pop(cmd: &SimpleContext) -> bool {
    git_subcommand_is(cmd, "stash", "pop")
}

fn is_stash_drop(cmd: &SimpleContext) -> bool {
    git_subcommand_is(cmd, "stash", "drop")
}

fn is_stash_clear(cmd: &SimpleContext) -> bool {
    git_subcommand_is(cmd, "stash", "clear")
}

fn is_clean_d(cmd: &SimpleContext) -> bool {
    let Some(ga) = parse_git_args(cmd) else {
        return false;
    };
    ga.args.first().is_some_and(|a| a == "clean")
        && ga
            .args
            .iter()
            .skip(1)
            .any(|arg| arg.starts_with('-') && !arg.starts_with("--") && arg.contains('d'))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    // === git reset --hard tests ===

    #[test]
    fn reset_hard() {
        assert_yaml_snapshot!(eval("git reset --hard"));
    }

    #[test]
    fn reset_hard_head() {
        assert_yaml_snapshot!(eval("git reset --hard HEAD"));
    }

    #[test]
    fn reset_hard_head_1() {
        assert_yaml_snapshot!(eval("git reset --hard HEAD~1"));
    }

    #[test]
    fn reset_hard_origin_main() {
        assert_yaml_snapshot!(eval("git reset --hard origin/main"));
    }

    #[test]
    fn chained_reset_hard() {
        assert_yaml_snapshot!(eval("git fetch && git reset --hard origin/main"));
    }

    #[test]
    fn reset_hard_in_chain() {
        assert_yaml_snapshot!(eval("git stash && git reset --hard && git stash pop"));
    }

    #[test]
    fn reset_passthrough() {
        assert_eq!(eval("git reset"), None);
    }

    #[test]
    fn reset_head_passthrough() {
        assert_eq!(eval("git reset HEAD"), None);
    }

    #[test]
    fn reset_soft_passthrough() {
        assert_eq!(eval("git reset --soft HEAD~1"), None);
    }

    #[test]
    fn reset_mixed_passthrough() {
        assert_eq!(eval("git reset --mixed HEAD~1"), None);
    }

    #[test]
    fn reset_file_passthrough() {
        assert_eq!(eval("git reset HEAD -- file.txt"), None);
    }

    #[test]
    fn git_status_passthrough() {
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_reset_hard_passthrough() {
        // echo is Allow via safe_rules
        let result = eval("echo git reset --hard is dangerous").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_reset_hard_passthrough() {
        // grep is Allow via safe_rules
        let result = eval("grep 'git reset --hard' README.md").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn c_path_reset_hard() {
        assert_yaml_snapshot!(eval("git -C /var/mnt/e/Repos/Rust/caesura reset --hard"));
    }

    #[test]
    fn c_path_reset_hard_head() {
        assert_yaml_snapshot!(eval(
            "git -C /var/mnt/e/Repos/Rust/caesura reset --hard HEAD~1"
        ));
    }

    #[test]
    fn c_path_quoted_reset_hard() {
        assert_yaml_snapshot!(eval(
            "git -C \"/var/mnt/e/Repos/Rust/caesura\" reset --hard"
        ));
    }

    #[test]
    fn c_path_reset_soft_passthrough() {
        assert_eq!(
            eval("git -C /var/mnt/e/Repos/Rust/caesura reset --soft HEAD~1"),
            None
        );
    }

    // === git stash tests ===

    #[test]
    fn stash_pop() {
        assert_yaml_snapshot!(eval("git stash pop"));
    }

    #[test]
    fn stash_pop_with_ref() {
        assert_yaml_snapshot!(eval("git stash pop stash@{0}"));
    }

    #[test]
    fn stash_pop_index() {
        assert_yaml_snapshot!(eval("git stash pop --index"));
    }

    #[test]
    fn chained_stash_pop() {
        assert_yaml_snapshot!(eval("git stash && git pull && git stash pop"));
    }

    #[test]
    fn stash_drop() {
        assert_yaml_snapshot!(eval("git stash drop"));
    }

    #[test]
    fn stash_drop_with_ref() {
        assert_yaml_snapshot!(eval("git stash drop stash@{0}"));
    }

    #[test]
    fn stash_drop_stash_2() {
        assert_yaml_snapshot!(eval("git stash drop stash@{2}"));
    }

    #[test]
    fn chained_stash_drop() {
        assert_yaml_snapshot!(eval("git stash list && git stash drop"));
    }

    #[test]
    fn stash_clear() {
        assert_yaml_snapshot!(eval("git stash clear"));
    }

    #[test]
    fn chained_stash_clear() {
        assert_yaml_snapshot!(eval("false || git stash clear"));
    }

    #[test]
    fn stash_passthrough() {
        assert_eq!(eval("git stash"), None);
    }

    #[test]
    fn stash_push_passthrough() {
        assert_eq!(eval("git stash push"), None);
    }

    #[test]
    fn stash_push_m_passthrough() {
        assert_eq!(eval("git stash push -m 'wip'"), None);
    }

    #[test]
    fn stash_apply_passthrough() {
        assert_eq!(eval("git stash apply"), None);
    }

    #[test]
    fn stash_apply_ref_passthrough() {
        assert_eq!(eval("git stash apply stash@{0}"), None);
    }

    #[test]
    fn stash_list_passthrough() {
        assert_eq!(eval("git stash list"), None);
    }

    #[test]
    fn stash_show_passthrough() {
        assert_eq!(eval("git stash show"), None);
    }

    #[test]
    fn stash_show_p_passthrough() {
        assert_eq!(eval("git stash show -p"), None);
    }

    #[test]
    fn stash_branch_passthrough() {
        assert_eq!(eval("git stash branch newbranch"), None);
    }

    #[test]
    fn echo_stash_pop_passthrough() {
        let result = eval("echo git stash pop is blocked").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_stash_drop_passthrough() {
        let result = eval("grep 'git stash drop' file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn cat_stash_clear_passthrough() {
        let result = eval("cat stash-clear-notes.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn c_path_stash_pop() {
        assert_yaml_snapshot!(eval("git -C /var/mnt/e/Repos/Rogue/docker/caddy stash pop"));
    }

    #[test]
    fn c_path_stash_drop() {
        assert_yaml_snapshot!(eval("git -C /var/mnt/e/Repos/Rust/caesura stash drop"));
    }

    #[test]
    fn c_path_stash_clear() {
        assert_yaml_snapshot!(eval("git -C /tmp/repo stash clear"));
    }

    #[test]
    fn c_path_quoted_stash_pop() {
        assert_yaml_snapshot!(eval("git -C \"/var/mnt/e/Repos/Rust/caesura\" stash pop"));
    }

    #[test]
    fn c_path_stash_apply_passthrough() {
        assert_eq!(
            eval("git -C /var/mnt/e/Repos/Rust/caesura stash apply"),
            None
        );
    }

    #[test]
    fn c_path_stash_passthrough() {
        assert_eq!(eval("git -C /var/mnt/e/Repos/Rust/caesura stash"), None);
    }

    // === git clean tests ===

    #[test]
    fn git_clean_fd() {
        assert_yaml_snapshot!(eval("git clean -fd"));
    }

    #[test]
    fn git_clean_fxd() {
        assert_yaml_snapshot!(eval("git clean -fxd"));
    }

    #[test]
    fn git_clean_d() {
        assert_yaml_snapshot!(eval("git clean -d"));
    }

    #[test]
    fn git_clean_df() {
        assert_yaml_snapshot!(eval("git clean -df"));
    }

    #[test]
    fn git_clean_dxf() {
        assert_yaml_snapshot!(eval("git clean -dxf"));
    }

    #[test]
    fn chained_git_clean_fd() {
        assert_yaml_snapshot!(eval("ls && git clean -fd"));
    }

    #[test]
    fn git_clean_f_passthrough() {
        assert_eq!(eval("git clean -f file.txt"), None);
    }

    #[test]
    fn git_clean_fx_passthrough() {
        assert_eq!(eval("git clean -fx file.txt"), None);
    }

    #[test]
    fn git_clean_fx_dash_in_filename_passthrough() {
        assert_eq!(
            eval("git clean -fx /path/to/some-dash-delimited-file.sh"),
            None
        );
    }

    #[test]
    fn git_clean_f_dash_in_path_passthrough() {
        assert_eq!(eval("git clean -f /path/dir-name/file.txt"), None);
    }

    #[test]
    fn git_clean_n_passthrough() {
        assert_eq!(eval("git clean -n"), None);
    }

    #[test]
    fn echo_git_clean_passthrough() {
        let result = eval("echo git clean -fxd").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn c_path_git_clean_fd() {
        assert_yaml_snapshot!(eval("git -C /var/mnt/e/Repos/Rust/caesura clean -fd"));
    }

    #[test]
    fn c_path_git_clean_fxd() {
        assert_yaml_snapshot!(eval("git -C /var/mnt/e/Repos/Rust/caesura clean -fxd"));
    }

    #[test]
    fn c_path_quoted_git_clean_fd() {
        assert_yaml_snapshot!(eval("git -C \"/var/mnt/e/Repos/Rust/caesura\" clean -fd"));
    }

    #[test]
    fn c_path_git_clean_f_passthrough() {
        assert_eq!(
            eval("git -C /var/mnt/e/Repos/Rust/caesura clean -f file.txt"),
            None
        );
    }
}
