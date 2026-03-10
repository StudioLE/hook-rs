//! Rule evaluation — matches parsed commands against registered rules.

use crate::prelude::*;

fn simple_rules() -> Vec<SimpleRule> {
    let mut rules = Vec::new();
    rules.push(rm_rule());
    rules.extend(find_rules());
    rules.extend(gh_rules());
    rules.extend(git_rules());
    rules.extend(git_approval_rules());
    rules.extend(git_checkout_rules());
    rules.extend(insta_rules());
    rules.extend(safe_rules());
    rules
}

fn complete_rules() -> Vec<CompleteRule> {
    let mut rules = Vec::new();
    rules.extend(cd_git_rules());
    rules.extend(chained_push_rules());
    rules.extend(echo_separator_rules());
    rules.extend(long_python_rules());
    rules
}

/// Merge an outcome into the accumulated result using Deny > Ask > Allow precedence.
fn merge(result: &mut Option<Outcome>, outcome: Outcome) {
    match outcome.decision {
        Decision::Deny => {
            if !result
                .as_ref()
                .is_some_and(|r| r.decision == Decision::Deny)
            {
                *result = Some(outcome);
            }
        }
        Decision::Ask => {
            if !result
                .as_ref()
                .is_some_and(|r| matches!(r.decision, Decision::Deny | Decision::Ask))
            {
                *result = Some(outcome);
            }
        }
        Decision::Allow => {
            if result.is_none() {
                *result = Some(outcome);
            }
        }
    }
}

/// Evaluate a shell command string against all registered rules.
///
/// Precedence: Deny > Ask > Allow.
/// For compound commands, all simple commands must be allowed for the result to be Allow.
#[must_use]
pub fn evaluate(command: &str) -> Option<Outcome> {
    let parsed = CompleteContext::parse(command)?;
    let complete_rules = complete_rules();
    let simple_rules = simple_rules();

    let mut result: Option<Outcome> = None;

    // CompleteRules evaluate cross-pipeline concerns.
    for rule in &complete_rules {
        if rule.matches(&parsed) {
            merge(&mut result, rule.outcome.clone());
        }
    }

    // SimpleRules run against each SimpleContext in the parsed command.
    for cmd in parsed.all_commands() {
        if let Some(outcome) = simple_rules
            .iter()
            .find_map(|rule| rule.matches(cmd).then(|| rule.outcome.clone()))
        {
            merge(&mut result, outcome);
        } else {
            // Command matched no rule — can't auto-allow the whole thing.
            if result
                .as_ref()
                .is_some_and(|r| r.decision == Decision::Allow)
            {
                result = None;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(command: &str) -> Option<Outcome> {
        evaluate(command)
    }

    #[test]
    fn safe_git_allowed() {
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn rm_denied() {
        let result = eval("rm -rf /tmp/nothing").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn stash_pop_denied() {
        let result = eval("git stash pop").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn reset_hard_denied() {
        let result = eval("git reset --hard").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn checkout_discard_denied() {
        let result = eval("git checkout -- file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn chained_push_denied() {
        let result = eval("git commit -m 'msg' && git push").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn echo_separator_denied() {
        let result = eval("cmd && echo \"---\"").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_denied() {
        let result = eval("find . -name '*.tmp' -delete").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn insta_heredoc_denied() {
        let result = eval("cargo insta review <<EOF\na\nEOF").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn cd_git_denied() {
        let result = eval("cd /path && git status").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn plain_ls_allowed() {
        let result = eval("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn plain_cargo_passthrough() {
        assert_eq!(eval("cargo build"), None);
    }

    #[test]
    fn standalone_push_passthrough() {
        assert_eq!(eval("git push"), None);
    }

    #[test]
    fn git_branch_read_allowed() {
        let result = eval("git branch -a").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_write_passthrough() {
        assert_eq!(eval("git branch -d old"), None);
    }

    #[test]
    fn git_tag_read_allowed() {
        let result = eval("git tag -l").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_create_passthrough() {
        assert_eq!(eval("git tag v1.0"), None);
    }

    #[test]
    fn git_remote_verbose_allowed() {
        let result = eval("git remote -v").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add_passthrough() {
        assert_eq!(eval("git remote add upstream https://x.com"), None);
    }

    #[test]
    fn tmp_rm_denied() {
        let result = eval("rm /tmp/file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_denied() {
        let result = eval("git clean -fd").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn forked_path_passthrough() {
        assert_eq!(eval("git -C /var/mnt/e/Repos/Forked/repo status"), None);
    }

    #[test]
    fn unknown_path_passthrough() {
        assert_eq!(eval("git -C /tmp/sketchy status"), None);
    }

    #[test]
    fn c_path_stash_pop_denied() {
        let result =
            eval("git -C /var/mnt/e/Repos/Rogue/docker/caddy stash pop").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("stash pop"));
    }

    #[test]
    fn c_path_reset_hard_denied() {
        let result =
            eval("git -C /var/mnt/e/Repos/Rust/caesura reset --hard").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("reset --hard"));
    }

    #[test]
    fn c_path_checkout_discard_denied() {
        let result = eval("git -C /var/mnt/e/Repos/Rust/caesura checkout -- file.txt")
            .expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("checkout --"));
    }

    #[test]
    fn c_path_git_clean_d_denied() {
        let result = eval("git -C /var/mnt/e/Repos/Rust/caesura clean -fd").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("clean"));
    }

    #[test]
    fn git_status_piped_allowed() {
        let result = eval("git status | head -5").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_diff_and_status_allowed() {
        let result = eval("git diff && git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn safe_and_unknown_passthrough() {
        assert_eq!(eval("git status && cargo build"), None);
    }
}
