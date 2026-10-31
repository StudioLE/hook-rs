//! Rules for `git push`.

use crate::prelude::*;

/// Git global options that take their value as the next argument.
const GIT_VALUE_OPTIONS: &[&str] = &[
    "-C",
    "-c",
    "--attr-source",
    "--config-env",
    "--git-dir",
    "--namespace",
    "--work-tree",
];

/// Deny `git push` when part of a compound command, and ask for every other `git push`.
pub fn git_push_rules() -> Vec<BashRule> {
    vec![git_push__chained(), git_push()]
}

/// Ask for `git push`, including after git global options like `-C <path>`.
///
/// - Checks no arguments, so a variable argument asks instead of the [`DenyReason::VariableArg`] deny
fn git_push() -> BashRule {
    BashRule {
        condition: Some(is_push),
        ..BashRule::new(
            "git_push",
            "git",
            Outcome::ask("`git push` requires approval"),
        )
    }
}

/// Is the first argument after git global options `push`?
fn is_push(ctx: &BashRuleContext) -> bool {
    let mut args = ctx.simple.args.iter();
    while let Some(arg) = args.next() {
        if GIT_VALUE_OPTIONS.contains(&arg.as_str()) {
            args.next();
        } else if !arg.starts_with('-') {
            return unquote_str(arg) == "push";
        }
    }
    false
}

/// Deny `git push` chained with other commands.
fn git_push__chained() -> BashRule {
    BashRule {
        condition: Some(|ctx| ctx.complete.is_chained()),
        ..BashRule::new(
            "git_push__chained",
            "git push",
            Outcome::deny("Chained `git push` is blocked. Run `git push` as a standalone command"),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn add_commit_push() {
        let result = eval_rules(
            git_push_rules(),
            "git add file.txt && git commit -m 'msg' && git push",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_push_no_space() {
        let result = eval_rules(git_push_rules(), "git commit -m 'msg'&& git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_push_with_remote() {
        let result = eval_rules(
            git_push_rules(),
            "git commit -m 'msg' && git push origin main",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn pull_push() {
        let result = eval_rules(git_push_rules(), "git pull && git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_or_push() {
        let result = eval_rules(git_push_rules(), "git commit -m 'msg' || git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn commit_semicolon_push() {
        let result = eval_rules(git_push_rules(), "git commit -m 'msg' ; git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn standalone_push() {
        let result = eval_rules(git_push_rules(), "git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_origin_main() {
        let result = eval_rules(git_push_rules(), "git push origin main");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_u() {
        let result = eval_rules(git_push_rules(), "git push -u origin feature-branch");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_set_upstream() {
        let result = eval_rules(git_push_rules(), "git push --set-upstream origin branch");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_force_with_lease() {
        let result = eval_rules(git_push_rules(), "git push --force-with-lease");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_force_with_lease_expect() {
        let result = eval_rules(
            git_push_rules(),
            "git push --force-with-lease=main:abc123 origin main",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_force() {
        let result = eval_rules(git_push_rules(), "git push -f");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_plus_refspec() {
        let result = eval_rules(git_push_rules(), "git push origin +main");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_delete() {
        let result = eval_rules(git_push_rules(), "git push --delete origin old");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    /// A variable argument asks instead of the [`DenyReason::VariableArg`] deny.
    #[test]
    fn push_variable() {
        let result = eval_rules(git_push_rules(), "git push origin \"$BRANCH\"");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn push_quoted_subcommand() {
        let result = eval_rules(git_push_rules(), "git \"push\"");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn c_path_push() {
        let result = eval_rules(git_push_rules(), "git -C /a/repo push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn no_pager_push() {
        let result = eval_rules(git_push_rules(), "git --no-pager push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn config_push() {
        let result = eval_rules(git_push_rules(), "git -c k=v push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn git_dir_push() {
        let result = eval_rules(git_push_rules(), "git --git-dir /a/.git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn git_dir_equals_push() {
        let result = eval_rules(git_push_rules(), "git --git-dir=/a/.git push");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn stash_push() {
        let result = eval_rules(git_push_rules(), "git stash push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn log_push_branch() {
        let result = eval_rules(git_push_rules(), "git log push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn commit_push_message() {
        let result = eval_rules(git_push_rules(), "git commit -m push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_status() {
        let result = eval_rules(git_push_rules(), "git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn commit_with_push_in_message() {
        let result = eval_rules(git_push_rules(), "git commit -m 'push changes'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_git_push() {
        let result = eval_rules(git_push_rules(), "echo git push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn push_with_substitution() {
        let result = eval_rules(
            git_push_rules(),
            "git push origin \"$(git branch --show-current)\"",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }
}
