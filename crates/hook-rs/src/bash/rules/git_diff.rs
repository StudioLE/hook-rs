//! Deny rule for `git diff` chained with other commands.

use crate::prelude::*;

/// Deny `git diff` when part of a compound command.
pub fn git_diff_rules() -> Vec<BashRule> {
    vec![git_diff__chained()]
}

/// Deny `git diff` and `git -C <path> diff` chained with other commands.
fn git_diff__chained() -> BashRule {
    BashRule {
        condition: Some(|ctx| ctx.complete.is_chained() && is_diff(&ctx.simple.args)),
        ..BashRule::new(
            "git_diff__chained",
            "git",
            Outcome::deny("Chained `git diff` is blocked. Run `git diff` as a standalone command"),
        )
    }
}

/// Is the git subcommand `diff`, skipping a leading `-C <path>`?
fn is_diff(args: &[String]) -> bool {
    let subcommand = match args.first() {
        Some(arg) if arg == "-C" => args.get(2),
        first => first,
    };
    subcommand.is_some_and(|arg| arg == "diff")
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn script_and_git_c_diff_stat() {
        let result = eval_rules(
            git_diff_rules(),
            "./generate-readmes.sh && git -C /home/user/repos/my-project diff --stat",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_diff_and_status() {
        let result = eval_rules(git_diff_rules(), "git diff && git status");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cargo_fmt_and_git_diff_stat() {
        let result = eval_rules(git_diff_rules(), "cargo fmt --all && git diff --stat");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_log_or_diff() {
        let result = eval_rules(git_diff_rules(), "git log || git diff");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn standalone_git_diff() {
        let result = eval_rules(git_diff_rules(), "git diff --stat");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn standalone_git_c_diff() {
        let result = eval_rules(
            git_diff_rules(),
            "git -C /home/user/repos/my-project diff --stat",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn piped_git_diff() {
        let result = eval_rules(git_diff_rules(), "git diff | wc -l");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn piped_git_c_diff() {
        let result = eval_rules(
            git_diff_rules(),
            "git -C /home/user/repos/my-project diff | wc -l",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_status_and_log() {
        let result = eval_rules(git_diff_rules(), "git status && git log");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// `diff` as the `-C` path is not the subcommand.
    #[test]
    fn git_c_path_named_diff() {
        let result = eval_rules(git_diff_rules(), "git -C diff status && git log");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
