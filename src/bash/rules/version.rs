//! Allow rule for bare `--version` on any command.

use crate::prelude::*;

/// Rules for bare `--version`.
pub fn version_rules() -> Vec<BashRule> {
    vec![any__version()]
}

/// Allow `<cmd> --version` with no subcommand or other flags.
fn any__version() -> BashRule {
    BashRule {
        condition: Some(is_bare_version),
        ..BashRule::new(
            "any__version",
            ANY_COMMAND,
            Outcome::allow("Bare `--version`"),
        )
    }
}

fn is_bare_version(ctx: &BashRuleContext) -> bool {
    ctx.simple.args == ["--version"] && is_plain_invocation(ctx, &[])
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn any__version_top_level() {
        let result = eval_rules(version_rules(), "aws --version");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn any__version_subcommand() {
        let result = eval_rules(version_rules(), "aws s3 --version");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__version_with_help() {
        let result = eval_rules(version_rules(), "aws --version --help");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__version_relative_path() {
        let result = eval_rules(version_rules(), "./x --version");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__version_package_runner() {
        let result = eval_rules(version_rules(), "npx --version");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn any__version_env_prefix() {
        let result = eval_rules(version_rules(), "FOO=1 aws --version");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
