//! Allow rule for bare `--help` on any command.

use crate::prelude::*;

/// Rules for bare `--help`.
pub fn help_rules() -> Vec<BashRule> {
    vec![any__help()]
}

/// Allow `<cmd> [sub…] --help` with `--help` last and no other flags.
fn any__help() -> BashRule {
    BashRule {
        condition: Some(is_bare_help),
        ..BashRule::new("any__help", ANY_COMMAND, Outcome::allow("Bare `--help`"))
    }
}

fn is_bare_help(ctx: &BashRuleContext) -> bool {
    let Some((last, components)) = ctx.simple.args.split_last() else {
        return false;
    };
    last == "--help" && is_plain_invocation(ctx, components)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn any__help_top_level() {
        let result = eval_rules(help_rules(), "aws --help");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn any__help_subcommand() {
        let result = eval_rules(help_rules(), "aws s3 --help");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn any__help_nested_subcommands() {
        let result = eval_rules(help_rules(), "gcloud compute instances list --help");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn any__help_underscore_and_dash() {
        let result = eval_rules(help_rules(), "my_tool sub-cmd --help");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn any__help_piped() {
        let result = eval_rules(help_rules(), "aws --help | aws s3 --help");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn any__help_bare_command() {
        let result = eval_rules(help_rules(), "aws");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_without_help() {
        let result = eval_rules(help_rules(), "aws s3 ls");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_with_version() {
        let result = eval_rules(help_rules(), "aws --version --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_with_flag() {
        let result = eval_rules(help_rules(), "aws -x --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_not_last() {
        let result = eval_rules(help_rules(), "aws s3 cp --help foo");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_relative_path() {
        let result = eval_rules(help_rules(), "./deploy --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_script_file() {
        let result = eval_rules(help_rules(), "foo.sh --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_url_subcommand() {
        let result = eval_rules(help_rules(), "aws s3://x --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_quoted_subcommand() {
        let result = eval_rules(help_rules(), "aws 'foo' --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_shell_script() {
        let result = eval_rules(help_rules(), "bash deploy --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_package_runner() {
        let result = eval_rules(help_rules(), "npx pkg --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_sudo() {
        let result = eval_rules(help_rules(), "sudo rm --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_env_prefix() {
        let result = eval_rules(help_rules(), "MANPAGER=x git log --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_variable() {
        let result = eval_rules(help_rules(), "aws $x --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__help_heredoc() {
        let result = eval_rules(help_rules(), "aws --help <<EOF\na\nEOF");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// The outer command is plain but runs a substitution first.
    #[test]
    fn any__help_substitution() {
        let result = eval_rules(help_rules(), "aws $(id) --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Checking a newly installed CLI with all real rules.
    #[test]
    fn any__help_chained_with_version() {
        let result =
            BashEvaluator::mock().evaluate_str("netlify --version && netlify login --help", None);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Existing deny rules still win over the bare `--help` allow.
    #[test]
    fn any__help_python_script() {
        let result = BashEvaluator::mock().evaluate_str("python script --help", None);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    /// `find` actions like `-exec` and `-delete` are flags, so they never reach the allow.
    #[test]
    fn any__help_find() {
        let result = BashEvaluator::mock().evaluate_str("find --help", None);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// `-exec` is a flag, so `find` would run `ls` but the allow never matches.
    #[test]
    fn any__help_find_exec() {
        let result = BashEvaluator::mock().evaluate_str("find -exec ls --help", None);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// `fd__ignored` denies `fd` without `--no-ignore`, and deny wins over the allow.
    #[test]
    fn any__help_fd() {
        let result = BashEvaluator::mock().evaluate_str("fd --help", None);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    /// `rg__ignored` denies `rg` without `--no-ignore`, and deny wins over the allow.
    #[test]
    fn any__help_rg() {
        let result = BashEvaluator::mock().evaluate_str("rg --help", None);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    /// `xargs` is excluded even with no operand, so it falls through to the prompt.
    #[test]
    fn any__help_xargs() {
        let result = BashEvaluator::mock().evaluate_str("xargs --help", None);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// `xargs` would pass `--help` to `rm`, which is why it is excluded.
    #[test]
    fn any__help_xargs_operand() {
        let result = BashEvaluator::mock().evaluate_str("xargs rm --help", None);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
