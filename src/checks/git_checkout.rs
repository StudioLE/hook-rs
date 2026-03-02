use regex::Regex;
use std::sync::LazyLock;

use crate::prelude::*;

static CHECKOUT_HEAD_DISCARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|&&|\|\||[;|])\s*git\s+checkout\s+HEAD\s+--").expect("valid regex")
});

static CHECKOUT_DISCARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|&&|\|\||[;|])\s*git\s+checkout\s+--\s").expect("valid regex")
});

pub fn check(command: &str) -> Option<CheckResult> {
    if CHECKOUT_HEAD_DISCARD.is_match(command) {
        return Some(CheckResult::deny(
            "git checkout HEAD -- is blocked. Do not discard changes to revert your mistakes. Fix the code properly.",
        ));
    }
    if CHECKOUT_DISCARD.is_match(command) {
        return Some(CheckResult::deny(
            "git checkout -- is blocked. Do not discard changes to revert your mistakes. Fix the code properly.",
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn checkout_head_file() {
        assert_yaml_snapshot!(check("git checkout HEAD -- file.txt"));
    }

    #[test]
    fn checkout_head_dot() {
        assert_yaml_snapshot!(check("git checkout HEAD -- ."));
    }

    #[test]
    fn checkout_head_src() {
        assert_yaml_snapshot!(check("git checkout HEAD -- src/"));
    }

    #[test]
    fn checkout_head_multiple() {
        assert_yaml_snapshot!(check("git checkout HEAD -- file1.txt file2.txt"));
    }

    #[test]
    fn chained_checkout_head() {
        assert_yaml_snapshot!(check("git status && git checkout HEAD -- file.txt"));
    }

    #[test]
    fn checkout_head_in_chain() {
        assert_yaml_snapshot!(check(
            "git stash && git checkout HEAD -- . && git stash pop"
        ));
    }

    #[test]
    fn checkout_double_dash_file() {
        assert_yaml_snapshot!(check("git checkout -- file.txt"));
    }

    #[test]
    fn checkout_double_dash_dot() {
        assert_yaml_snapshot!(check("git checkout -- ."));
    }

    #[test]
    fn checkout_double_dash_src() {
        assert_yaml_snapshot!(check("git checkout -- src/"));
    }

    #[test]
    fn chained_checkout_double_dash() {
        assert_yaml_snapshot!(check("git status && git checkout -- file.txt"));
    }

    #[test]
    fn checkout_branch_passthrough() {
        assert_eq!(check("git checkout main"), None);
    }

    #[test]
    fn checkout_b_passthrough() {
        assert_eq!(check("git checkout -b new-branch"), None);
    }

    #[test]
    fn checkout_head_1_passthrough() {
        assert_eq!(check("git checkout HEAD~1"), None);
    }

    #[test]
    fn checkout_head_caret_passthrough() {
        assert_eq!(check("git checkout HEAD^"), None);
    }

    #[test]
    fn git_status_passthrough() {
        assert_eq!(check("git status"), None);
    }

    #[test]
    fn echo_checkout_head_passthrough() {
        assert_eq!(check("echo git checkout HEAD -- is dangerous"), None);
    }

    #[test]
    fn echo_checkout_discard_passthrough() {
        assert_eq!(check("echo git checkout -- is dangerous"), None);
    }

    #[test]
    fn grep_checkout_head_passthrough() {
        assert_eq!(check("grep 'git checkout HEAD --' README.md"), None);
    }

    #[test]
    fn grep_checkout_discard_passthrough() {
        assert_eq!(check("grep 'git checkout --' README.md"), None);
    }
}
