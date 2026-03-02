use crate::command;
use crate::prelude::*;

#[must_use]
pub fn check(command: &str) -> Option<CheckResult> {
    for args in command::git_args_in_segments(command) {
        let mut parts = args.split_whitespace();
        if parts.next() != Some("stash") {
            continue;
        }
        match parts.next() {
            Some("pop") => {
                return Some(CheckResult::deny(
                    "git stash pop is blocked. Use 'git stash apply' instead to keep the stash entry for safety.",
                ));
            }
            Some("drop") => {
                return Some(CheckResult::deny(
                    "git stash drop is blocked. Use 'git stash list' to view stashes, 'git stash show' to inspect them.",
                ));
            }
            Some("clear") => {
                return Some(CheckResult::deny(
                    "git stash clear is blocked. This would delete all stashes permanently.",
                ));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn stash_pop() {
        assert_yaml_snapshot!(check("git stash pop"));
    }

    #[test]
    fn stash_pop_with_ref() {
        assert_yaml_snapshot!(check("git stash pop stash@{0}"));
    }

    #[test]
    fn stash_pop_index() {
        assert_yaml_snapshot!(check("git stash pop --index"));
    }

    #[test]
    fn chained_stash_pop() {
        assert_yaml_snapshot!(check("git stash && git pull && git stash pop"));
    }

    #[test]
    fn stash_drop() {
        assert_yaml_snapshot!(check("git stash drop"));
    }

    #[test]
    fn stash_drop_with_ref() {
        assert_yaml_snapshot!(check("git stash drop stash@{0}"));
    }

    #[test]
    fn stash_drop_stash_2() {
        assert_yaml_snapshot!(check("git stash drop stash@{2}"));
    }

    #[test]
    fn chained_stash_drop() {
        assert_yaml_snapshot!(check("git stash list && git stash drop"));
    }

    #[test]
    fn stash_clear() {
        assert_yaml_snapshot!(check("git stash clear"));
    }

    #[test]
    fn chained_stash_clear() {
        assert_yaml_snapshot!(check("false || git stash clear"));
    }

    #[test]
    fn stash_passthrough() {
        assert_eq!(check("git stash"), None);
    }

    #[test]
    fn stash_push_passthrough() {
        assert_eq!(check("git stash push"), None);
    }

    #[test]
    fn stash_push_m_passthrough() {
        assert_eq!(check("git stash push -m 'wip'"), None);
    }

    #[test]
    fn stash_apply_passthrough() {
        assert_eq!(check("git stash apply"), None);
    }

    #[test]
    fn stash_apply_ref_passthrough() {
        assert_eq!(check("git stash apply stash@{0}"), None);
    }

    #[test]
    fn stash_list_passthrough() {
        assert_eq!(check("git stash list"), None);
    }

    #[test]
    fn stash_show_passthrough() {
        assert_eq!(check("git stash show"), None);
    }

    #[test]
    fn stash_show_p_passthrough() {
        assert_eq!(check("git stash show -p"), None);
    }

    #[test]
    fn stash_branch_passthrough() {
        assert_eq!(check("git stash branch newbranch"), None);
    }

    #[test]
    fn git_status_passthrough() {
        assert_eq!(check("git status"), None);
    }

    #[test]
    fn echo_stash_pop_passthrough() {
        assert_eq!(check("echo git stash pop is blocked"), None);
    }

    #[test]
    fn grep_stash_drop_passthrough() {
        assert_eq!(check("grep 'git stash drop' file.txt"), None);
    }

    #[test]
    fn cat_stash_clear_passthrough() {
        assert_eq!(check("cat stash-clear-notes.txt"), None);
    }

    #[test]
    fn c_path_stash_pop() {
        assert_yaml_snapshot!(check(
            "git -C /var/mnt/e/Repos/Rogue/docker/caddy stash pop"
        ));
    }

    #[test]
    fn c_path_stash_drop() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura stash drop"));
    }

    #[test]
    fn c_path_stash_clear() {
        assert_yaml_snapshot!(check("git -C /tmp/repo stash clear"));
    }

    #[test]
    fn c_path_quoted_stash_pop() {
        assert_yaml_snapshot!(check("git -C \"/var/mnt/e/Repos/Rust/caesura\" stash pop"));
    }

    #[test]
    fn c_path_stash_apply_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura stash apply"),
            None
        );
    }

    #[test]
    fn c_path_stash_passthrough() {
        assert_eq!(check("git -C /var/mnt/e/Repos/Rust/caesura stash"), None);
    }
}
