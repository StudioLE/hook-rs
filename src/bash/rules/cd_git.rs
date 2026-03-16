//! Deny rule for `cd <path> && git <cmd>` patterns.

use crate::prelude::*;

/// Deny `cd` chained with `git`, directing to `git -C <path>` instead.
pub fn cd_git_rules() -> Vec<BashRule> {
    vec![cd_git()]
}

/// Deny `cd` chained with `git`.
fn cd_git() -> BashRule {
    BashRule {
        condition: Some(is_cd_then_git),
        ..BashRule::new(
            "cd_git",
            "cd",
            Outcome::deny("Do not chain cd and git. Use 'git -C <path> <command>' instead."),
        )
    }
}

fn is_cd_then_git(_cmd: &SimpleContext, complete: &CompleteContext, _settings: &Settings) -> bool {
    let mut seen_cd = false;
    for pipeline in &complete.children {
        let Some(first) = pipeline.children.first() else {
            continue;
        };
        if first.name == "cd" {
            seen_cd = true;
        } else if seen_cd && first.name == "git" {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn cd_and_git_status() {
        let outcome = evaluate_expect_outcome("cd /home/user/repos/my-project && git status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn cd_and_git_commit() {
        let outcome =
            evaluate_expect_outcome("cd /home/user/repos/my-project && git commit -m 'msg'");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn cd_untrusted_and_git() {
        let outcome = evaluate_expect_outcome("cd /tmp/sketchy && git log");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn cd_relative_and_git() {
        let outcome = evaluate_expect_outcome("cd ../relative/path && git diff");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn cd_forked_and_git() {
        let outcome = evaluate_expect_outcome("cd /home/user/repos/forked/repo && git status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn cd_semicolon_git() {
        let outcome = evaluate_expect_outcome("cd /path ; git status");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_or_git() {
        let outcome = evaluate_expect_outcome("cd /path || git status");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_cmd_git() {
        let outcome = evaluate_expect_outcome("cd /path && ls && git status");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_multiple_git() {
        let outcome = evaluate_expect_outcome("cd /path && git fetch && git rebase origin/main");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_then_cd_passthrough() {
        let reason = evaluate_expect_skip("git status && cd /path");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn cd_alone_passthrough() {
        let reason = evaluate_expect_skip("cd /home/user/repos/my-project");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_alone_passthrough() {
        // git status alone is matched by git_approval as Allow, not passthrough
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_log_passthrough() {
        // git log is matched by git_approval as Allow
        let outcome = evaluate_expect_outcome("git log --oneline -5");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn non_cd_compound_passthrough() {
        // ls -la is Allow via safe_rules, git status is Allow via git_approval
        let outcome = evaluate_expect_outcome("ls -la && git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_cd_allowed() {
        // echo is Allow via safe_rules, git status is Allow via git_approval
        let outcome = evaluate_expect_outcome("echo cd /path && git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
