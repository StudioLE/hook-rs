//! Deny rule for `git push` chained with other commands.

use crate::prelude::*;

/// Deny `git push` when part of a compound command.
pub fn chained_push_rules() -> Vec<BashRule> {
    vec![git_push__chained()]
}

/// Deny `git push` chained with other commands.
fn git_push__chained() -> BashRule {
    BashRule {
        condition: Some(is_chained),
        ..BashRule::new(
            "git_push__chained",
            "git push",
            Outcome::deny("Chained `git push` is blocked. Run `git push` as a standalone command"),
        )
    }
}

fn is_chained(ctx: &BashRuleContext) -> bool {
    ctx.complete.children.len() > 1
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn add_commit_push() {
        let result = eval_rules(
            chained_push_rules(),
            "git add file.txt && git commit -m 'msg' && git push",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_push_no_space() {
        let result = eval_rules(chained_push_rules(), "git commit -m 'msg'&& git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_push_with_remote() {
        let result = eval_rules(
            chained_push_rules(),
            "git commit -m 'msg' && git push origin main",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn pull_push() {
        let result = eval_rules(chained_push_rules(), "git pull && git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_or_push() {
        let result = eval_rules(chained_push_rules(), "git commit -m 'msg' || git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_semicolon_push() {
        let result = eval_rules(chained_push_rules(), "git commit -m 'msg' ; git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn standalone_push() {
        let result = eval_rules(chained_push_rules(), "git push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_origin_main() {
        let result = eval_rules(chained_push_rules(), "git push origin main");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_u() {
        let result = eval_rules(chained_push_rules(), "git push -u origin feature-branch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_set_upstream() {
        let result = eval_rules(
            chained_push_rules(),
            "git push --set-upstream origin branch",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_force_with_lease() {
        let result = eval_rules(chained_push_rules(), "git push --force-with-lease");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_status() {
        let result = eval_rules(chained_push_rules(), "git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn commit_with_push_in_message() {
        let result = eval_rules(chained_push_rules(), "git commit -m 'push changes'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_git_push() {
        let result = eval_rules(chained_push_rules(), "echo git push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_with_substitution() {
        let result = eval_rules(
            chained_push_rules(),
            "git push origin \"$(git branch --show-current)\"",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
