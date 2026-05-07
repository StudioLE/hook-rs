//! Allow rule for `cd <path>` when path matches read globs in settings.

use crate::prelude::*;

/// Allow `cd` to trusted paths matching `settings.read.paths`.
pub fn cd_rules() -> Vec<BashRule> {
    vec![cd_trusted_path()]
}

/// Allow `cd <absolute-path>` when the path matches read globs.
fn cd_trusted_path() -> BashRule {
    BashRule {
        id: "cd__trusted_path".to_owned(),
        command: "cd".to_owned(),
        condition: Some(is_cd_path_trusted),
        outcome: Outcome::allow("cd to trusted path"),
        ..Default::default()
    }
}

/// True if `cd` has exactly one operand matching a trusted read path.
fn is_cd_path_trusted(ctx: &BashRuleContext) -> bool {
    let Ok(parsed) = parse_cd_args(&ctx.simple.args) else {
        return false;
    };
    let Some(cd) = parsed.first() else {
        return false;
    };
    let Some(path) = cd.operands.first() else {
        return false;
    };
    if let Some(is_allowed) = ctx.paths.is_match(path, &ctx.settings.read.paths) {
        trace!(is_allowed, "Matched cd path");
        return is_allowed;
    }
    trace!("No cd path match");
    false
}

/// Parse `cd` args: no options, one required absolute path operand.
fn parse_cd_args(args: &[String]) -> Result<ParsedCommand, Report<CommandParseError>> {
    let schema = CommandSchemaBuilder::new("cd")
        .with_operand(
            OperandSchemaBuilder::new("path")
                .with_value(ValueConstraint::glob("/**").expect("/** is a valid glob"))
                .build(),
        )
        .build();
    CommandParser::new(schema).parse(args.to_vec())
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn trusted_path() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let outcome = eval_outcome("cd /a/repos/project", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn untrusted_path() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let reason = eval_skip("cd /tmp/sketchy", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn negated_path() {
        let settings = Settings::with_read(&["/a/**", "!/a/secret/**"]);
        let reason = eval_skip("cd /a/secret/dir", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn relative_path() {
        let settings = Settings::with_read(&["./**"]);
        let reason = eval_skip("cd ./subdir", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn no_patterns() {
        let settings = Settings::with_read(&[]);
        let reason = eval_skip("cd /a/repos/project", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn with_flags_rejected() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let reason = eval_skip("cd -P /a/repos/project", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// `cd` to a trusted path followed by `gh api` should allow the full chain.
    #[test]
    fn chained_with_gh_api() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let outcome = eval_outcome("cd /a/repos/project && gh api repos/owner/repo", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// `cd` to a trusted path followed by `cargo test` should allow.
    #[test]
    fn chained_with_cargo() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let outcome = eval_outcome("cd /a/repos/project && cargo test", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Mock settings should allow `cd` to paths covered by read globs.
    #[test]
    fn mock_settings() {
        let outcome = evaluate_expect_outcome("cd /path/to/repos/project");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    fn eval_outcome(command: &str, settings: Settings) -> Outcome {
        let _logger = init_test_logger();
        eval(command, settings).expect("command should produce an outcome")
    }

    #[expect(clippy::panic, reason = "test helper")]
    fn eval_skip(command: &str, settings: Settings) -> SkipReason {
        let _logger = init_test_logger();
        match eval(command, settings)
            .expect_err("command should not succeed")
            .current_context()
        {
            ParseError::Skip(reason) => *reason,
            other => panic!("expected Skip, got {other:?}"),
        }
    }
}
