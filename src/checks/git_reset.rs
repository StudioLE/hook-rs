use regex::Regex;
use std::sync::LazyLock;

use crate::prelude::*;

static RESET_HARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|&&|\|\||[;|])\s*git\s+reset\s+.*--hard(?:\s|$)").expect("valid regex")
});

pub fn check(command: &str) -> Option<CheckResult> {
    RESET_HARD.is_match(command).then(|| {
        CheckResult::deny(
            "git reset --hard is blocked. This discards all uncommitted changes permanently.",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

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
}
