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
            Outcome::deny(
                "`cd ... && git ...` is blocked. Alternatives: `git -C <path> <command>`",
            ),
        )
    }
}

fn is_cd_then_git(ctx: &BashRuleContext) -> bool {
    let mut seen_cd = false;
    for pipeline in &ctx.complete.children {
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

    #[test]
    fn cd_and_git_status() {
        let result = eval_rules(
            cd_git_rules(),
            "cd /home/user/repos/my-project && git status",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_and_git_commit() {
        let result = eval_rules(
            cd_git_rules(),
            "cd /home/user/repos/my-project && git commit -m 'msg'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_untrusted_and_git() {
        let result = eval_rules(cd_git_rules(), "cd /tmp/sketchy && git log");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_relative_and_git() {
        let result = eval_rules(cd_git_rules(), "cd ../relative/path && git diff");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_forked_and_git() {
        let result = eval_rules(
            cd_git_rules(),
            "cd /home/user/repos/forked/repo && git status",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_semicolon_git() {
        let result = eval_rules(cd_git_rules(), "cd /path ; git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_or_git() {
        let result = eval_rules(cd_git_rules(), "cd /path || git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_cmd_git() {
        let result = eval_rules(cd_git_rules(), "cd /path && ls && git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cd_multiple_git() {
        let result = eval_rules(
            cd_git_rules(),
            "cd /path && git fetch && git rebase origin/main",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_then_cd() {
        let result = eval_rules(cd_git_rules(), "git status && cd /path");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cd_alone() {
        let result = eval_rules(cd_git_rules(), "cd /home/user/repos/my-project");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_alone() {
        let result = eval_rules(cd_git_rules(), "git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_log() {
        let result = eval_rules(cd_git_rules(), "git log --oneline -5");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn non_cd_compound() {
        let result = eval_rules(cd_git_rules(), "ls -la && git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_cd_compound() {
        let result = eval_rules(cd_git_rules(), "echo cd /path && git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
