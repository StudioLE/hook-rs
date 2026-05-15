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
        let result = eval_rules_with_settings(cd_rules(), "cd /a/repos/project", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn untrusted_path() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let result = eval_rules_with_settings(cd_rules(), "cd /tmp/sketchy", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn negated_path() {
        let settings = Settings::with_read(&["/a/**", "!/a/secret/**"]);
        let result = eval_rules_with_settings(cd_rules(), "cd /a/secret/dir", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn relative_path() {
        let settings = Settings::with_read(&["./**"]);
        let result = eval_rules_with_settings(cd_rules(), "cd ./subdir", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn no_patterns() {
        let settings = Settings::with_read(&[]);
        let result = eval_rules_with_settings(cd_rules(), "cd /a/repos/project", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn with_flags_rejected() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let result = eval_rules_with_settings(cd_rules(), "cd -P /a/repos/project", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// `cd` to a trusted path followed by an unmatched command.
    #[test]
    fn chained_with_gh_api() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let result = eval_rules_with_settings(
            cd_rules(),
            "cd /a/repos/project && gh api repos/owner/repo",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    /// `cd` to a trusted path followed by an unmatched command.
    #[test]
    fn chained_with_cargo() {
        let settings = Settings::with_read(&["/a/repos/**"]);
        let result =
            eval_rules_with_settings(cd_rules(), "cd /a/repos/project && cargo test", settings);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    /// Mock settings should allow `cd` to paths covered by read globs.
    #[test]
    fn mock_settings() {
        let result = eval_rules(cd_rules(), "cd /path/to/repos/project");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
