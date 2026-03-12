//! Deny rule for `git push` chained with other commands.

use crate::prelude::*;

/// Deny `git push` when part of a compound command.
pub fn chained_push_rules() -> Vec<CompleteRule> {
    vec![git_push__chained()]
}

/// Deny `git push` chained with other commands.
fn git_push__chained() -> CompleteRule {
    CompleteRule {
        id: "git_push__chained".to_owned(),
        condition: Some(is_chained_git_push),
        outcome: Outcome::deny(
            "Chained git push is blocked. Run 'git push' as a separate, standalone command.",
        ),
    }
}

fn is_chained_git_push(parsed: &CompleteContext, _settings: &Settings) -> bool {
    let has_git_push = parsed
        .all_commands()
        .any(|cmd| cmd.name == "git" && cmd.args.first().is_some_and(|a| a == "push"));
    has_git_push && parsed.children.len() > 1
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn add_commit_push() {
        let outcome =
            evaluate_expect_outcome("git add file.txt && git commit -m 'msg' && git push");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn commit_push_no_space() {
        let outcome = evaluate_expect_outcome("git commit -m 'msg'&& git push");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn commit_push_with_remote() {
        let outcome = evaluate_expect_outcome("git commit -m 'msg' && git push origin main");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn pull_push() {
        let outcome = evaluate_expect_outcome("git pull && git push");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn commit_or_push() {
        let outcome = evaluate_expect_outcome("git commit -m 'msg' || git push");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn commit_semicolon_push() {
        let outcome = evaluate_expect_outcome("git commit -m 'msg' ; git push");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn standalone_push_passthrough() {
        let reason = evaluate_expect_skip("git push");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_origin_main_passthrough() {
        let reason = evaluate_expect_skip("git push origin main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_u_passthrough() {
        let reason = evaluate_expect_skip("git push -u origin feature-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_set_upstream_passthrough() {
        let reason = evaluate_expect_skip("git push --set-upstream origin branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_force_with_lease_passthrough() {
        let reason = evaluate_expect_skip("git push --force-with-lease");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_status_passthrough() {
        // git status alone is Allow via git_approval, not passthrough
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn commit_with_push_in_message_passthrough() {
        let reason = evaluate_expect_skip("git commit -m 'push changes'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_git_push_passthrough() {
        // echo is Allow via safe_rules
        let outcome = evaluate_expect_outcome("echo git push");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn push_with_substitution_passthrough() {
        let reason = evaluate_expect_skip("git push origin \"$(git branch --show-current)\"");
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
