//! Check for `git reset --hard` commands.

use crate::prelude::*;

/// Deny `git reset --hard` commands.
#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    for cmd in parsed.all_commands() {
        let Some(ga) = parse_git_args(cmd) else {
            continue;
        };
        if ga.args.first().is_some_and(|a| a == "reset") && ga.args.iter().any(|a| a == "--hard") {
            return Some(CheckResult::deny(
                "git reset --hard is blocked. This discards all uncommitted changes permanently.",
            ));
        }
    }
    None
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
    fn reset_hard() {
        assert_yaml_snapshot!(check("git reset --hard"));
    }

    #[test]
    fn reset_hard_head() {
        assert_yaml_snapshot!(check("git reset --hard HEAD"));
    }

    #[test]
    fn reset_hard_head_1() {
        assert_yaml_snapshot!(check("git reset --hard HEAD~1"));
    }

    #[test]
    fn reset_hard_origin_main() {
        assert_yaml_snapshot!(check("git reset --hard origin/main"));
    }

    #[test]
    fn chained_reset_hard() {
        assert_yaml_snapshot!(check("git fetch && git reset --hard origin/main"));
    }

    #[test]
    fn reset_hard_in_chain() {
        assert_yaml_snapshot!(check("git stash && git reset --hard && git stash pop"));
    }

    #[test]
    fn reset_passthrough() {
        assert_eq!(check("git reset"), None);
    }

    #[test]
    fn reset_head_passthrough() {
        assert_eq!(check("git reset HEAD"), None);
    }

    #[test]
    fn reset_soft_passthrough() {
        assert_eq!(check("git reset --soft HEAD~1"), None);
    }

    #[test]
    fn reset_mixed_passthrough() {
        assert_eq!(check("git reset --mixed HEAD~1"), None);
    }

    #[test]
    fn reset_file_passthrough() {
        assert_eq!(check("git reset HEAD -- file.txt"), None);
    }

    #[test]
    fn git_status_passthrough() {
        assert_eq!(check("git status"), None);
    }

    #[test]
    fn echo_reset_hard_passthrough() {
        assert_eq!(check("echo git reset --hard is dangerous"), None);
    }

    #[test]
    fn grep_reset_hard_passthrough() {
        assert_eq!(check("grep 'git reset --hard' README.md"), None);
    }

    #[test]
    fn c_path_reset_hard() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura reset --hard"));
    }

    #[test]
    fn c_path_reset_hard_head() {
        assert_yaml_snapshot!(check(
            "git -C /var/mnt/e/Repos/Rust/caesura reset --hard HEAD~1"
        ));
    }

    #[test]
    fn c_path_quoted_reset_hard() {
        assert_yaml_snapshot!(check(
            "git -C \"/var/mnt/e/Repos/Rust/caesura\" reset --hard"
        ));
    }

    #[test]
    fn c_path_reset_soft_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura reset --soft HEAD~1"),
            None
        );
    }
}
