use crate::prelude::*;
use crate::utils::git::parse_git_args;

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

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn checkout_head_file() {
        assert_yaml_snapshot!(eval("git checkout HEAD -- file.txt"));
    }

    #[test]
    fn checkout_head_dot() {
        assert_yaml_snapshot!(eval("git checkout HEAD -- ."));
    }

    #[test]
    fn checkout_head_src() {
        assert_yaml_snapshot!(eval("git checkout HEAD -- src/"));
    }

    #[test]
    fn checkout_head_multiple() {
        assert_yaml_snapshot!(eval("git checkout HEAD -- file1.txt file2.txt"));
    }

    #[test]
    fn chained_checkout_head() {
        assert_yaml_snapshot!(eval("git status && git checkout HEAD -- file.txt"));
    }

    #[test]
    fn checkout_head_in_chain() {
        assert_yaml_snapshot!(eval("git stash && git checkout HEAD -- . && git stash pop"));
    }

    #[test]
    fn checkout_double_dash_file() {
        assert_yaml_snapshot!(eval("git checkout -- file.txt"));
    }

    #[test]
    fn checkout_double_dash_dot() {
        assert_yaml_snapshot!(eval("git checkout -- ."));
    }

    #[test]
    fn checkout_double_dash_src() {
        assert_yaml_snapshot!(eval("git checkout -- src/"));
    }

    #[test]
    fn chained_checkout_double_dash() {
        assert_yaml_snapshot!(eval("git status && git checkout -- file.txt"));
    }

    #[test]
    fn checkout_branch_passthrough() {
        assert_eq!(eval("git checkout main"), None);
    }

    #[test]
    fn checkout_b_passthrough() {
        assert_eq!(eval("git checkout -b new-branch"), None);
    }

    #[test]
    fn checkout_head_1_passthrough() {
        assert_eq!(eval("git checkout HEAD~1"), None);
    }

    #[test]
    fn checkout_head_caret_passthrough() {
        assert_eq!(eval("git checkout HEAD^"), None);
    }

    #[test]
    fn git_status_passthrough() {
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_checkout_head_passthrough() {
        let result = eval("echo git checkout HEAD -- is dangerous").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_checkout_discard_passthrough() {
        let result = eval("echo git checkout -- is dangerous").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_checkout_head_passthrough() {
        let result = eval("grep 'git checkout HEAD --' README.md").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_checkout_discard_passthrough() {
        let result = eval("grep 'git checkout --' README.md").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn c_path_checkout_head_file() {
        assert_yaml_snapshot!(eval(
            "git -C /var/mnt/e/Repos/Rust/caesura checkout HEAD -- file.txt"
        ));
    }

    #[test]
    fn c_path_checkout_discard() {
        assert_yaml_snapshot!(eval(
            "git -C /var/mnt/e/Repos/Rust/caesura checkout -- file.txt"
        ));
    }

    #[test]
    fn c_path_quoted_checkout_discard() {
        assert_yaml_snapshot!(eval(
            "git -C \"/var/mnt/e/Repos/Rust/caesura\" checkout -- ."
        ));
    }

    #[test]
    fn c_path_checkout_branch_passthrough() {
        assert_eq!(
            eval("git -C /var/mnt/e/Repos/Rust/caesura checkout main"),
            None
        );
    }
}
