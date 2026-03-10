//! Deny rules for `git checkout` that discards uncommitted changes.

use crate::prelude::*;
use crate::utils::git::parse_git_args;

/// Deny `git checkout -- <path>` and `git checkout HEAD -- <path>`.
pub fn git_checkout_rules() -> Vec<SimpleRule> {
    vec![
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_checkout_head_discard),
            outcome: Outcome::deny(
                "git checkout HEAD -- is blocked. Do not discard changes to revert your mistakes. \
                 Fix the code properly.",
            ),
            ..Default::default()
        },
        SimpleRule {
            prefix: "git".to_owned(),
            condition: Some(is_checkout_discard),
            outcome: Outcome::deny(
                "git checkout -- is blocked. Do not discard changes to revert your mistakes. \
                 Fix the code properly.",
            ),
            ..Default::default()
        },
    ]
}

fn is_checkout_head_discard(cmd: &SimpleContext) -> bool {
    let Some(ga) = parse_git_args(cmd) else {
        return false;
    };
    ga.args.first().is_some_and(|a| a == "checkout")
        && ga.args.get(1).is_some_and(|a| a == "HEAD")
        && ga.args.get(2).is_some_and(|a| a == "--")
}

fn is_checkout_discard(cmd: &SimpleContext) -> bool {
    let Some(ga) = parse_git_args(cmd) else {
        return false;
    };
    ga.args.first().is_some_and(|a| a == "checkout")
        && ga.args.get(1).is_some_and(|a| a == "--")
        && ga.args.len() > 2
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn checkout_head_file() {
        let result = evaluate("git checkout HEAD -- file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_head_dot() {
        let result = evaluate("git checkout HEAD -- .");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_head_src() {
        let result = evaluate("git checkout HEAD -- src/");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_head_multiple() {
        let result = evaluate("git checkout HEAD -- file1.txt file2.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn chained_checkout_head() {
        let result = evaluate("git status && git checkout HEAD -- file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_head_in_chain() {
        let result = evaluate("git stash && git checkout HEAD -- . && git stash pop");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_double_dash_file() {
        let result = evaluate("git checkout -- file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_double_dash_dot() {
        let result = evaluate("git checkout -- .");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_double_dash_src() {
        let result = evaluate("git checkout -- src/");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn chained_checkout_double_dash() {
        let result = evaluate("git status && git checkout -- file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn checkout_branch_passthrough() {
        assert_eq!(evaluate("git checkout main"), None);
    }

    #[test]
    fn checkout_b_passthrough() {
        assert_eq!(evaluate("git checkout -b new-branch"), None);
    }

    #[test]
    fn checkout_head_1_passthrough() {
        assert_eq!(evaluate("git checkout HEAD~1"), None);
    }

    #[test]
    fn checkout_head_caret_passthrough() {
        assert_eq!(evaluate("git checkout HEAD^"), None);
    }

    #[test]
    fn git_status_passthrough() {
        let result = evaluate("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_checkout_head_passthrough() {
        let result = evaluate("echo git checkout HEAD -- is dangerous").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_checkout_discard_passthrough() {
        let result = evaluate("echo git checkout -- is dangerous").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_checkout_head_passthrough() {
        let result = evaluate("grep 'git checkout HEAD --' README.md").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_checkout_discard_passthrough() {
        let result = evaluate("grep 'git checkout --' README.md").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn c_path_checkout_head_file() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura checkout HEAD -- file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn c_path_checkout_discard() {
        let result = evaluate("git -C /var/mnt/e/Repos/Rust/caesura checkout -- file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn c_path_quoted_checkout_discard() {
        let result = evaluate("git -C \"/var/mnt/e/Repos/Rust/caesura\" checkout -- .");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn c_path_checkout_branch_passthrough() {
        assert_eq!(
            evaluate("git -C /var/mnt/e/Repos/Rust/caesura checkout main"),
            None
        );
    }
}
