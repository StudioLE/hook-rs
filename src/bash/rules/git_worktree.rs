//! Deny and allow rules for `git worktree add` in trusted paths.

use crate::prelude::*;

/// Deny and allow rules for `git worktree add` in trusted paths.
pub fn git_worktree_rules() -> Vec<BashRule> {
    vec![
        git_worktree_add__unsupported_flags(),
        git_worktree_add__trusted_path(),
    ]
}

/// Deny `git worktree add` with any flag other than `-b`.
fn git_worktree_add__unsupported_flags() -> BashRule {
    BashRule {
        id: "git_worktree_add__unsupported_flags".to_owned(),
        command: "git worktree add".to_owned(),
        condition: Some(has_unsupported_flags),
        outcome: Outcome::deny(
            "Use `git worktree add <path>` or `git worktree add -b <branch> <path>`",
        ),
        ..Default::default()
    }
}

/// Allow `git worktree add` when the target path is in a trusted directory.
fn git_worktree_add__trusted_path() -> BashRule {
    BashRule {
        id: "git_worktree_add__trusted_path".to_owned(),
        command: "git worktree add".to_owned(),
        condition: Some(is_worktree_path_trusted),
        outcome: Outcome::allow("Safe `git worktree add` in trusted path"),
        ..Default::default()
    }
}

/// True if any flag other than `-b` is present in the args.
fn has_unsupported_flags(ctx: &BashRuleContext) -> bool {
    parse_worktree_args(&ctx.simple.args).is_err()
}

/// True if the path operand matches a trusted worktree path.
///
/// The [`CommandParser`] schema constrains the path operand to
/// absolute paths via `/**` glob, so no manual prefix check is needed.
fn is_worktree_path_trusted(ctx: &BashRuleContext) -> bool {
    let Ok(parsed) = parse_worktree_args(&ctx.simple.args) else {
        return false;
    };
    let Some(add) = parsed.get(2) else {
        return false;
    };
    let Some(path) = add.operands.first() else {
        return false;
    };
    if let Some(is_allowed) = ctx.paths.is_match(path, &ctx.settings.worktrees.paths) {
        trace!(is_allowed, "Matched worktree path");
        return is_allowed;
    }
    trace!("No worktree path match");
    false
}

/// Parse `git worktree add` args using a schema that only allows `-b`.
///
/// Constrains the path operand to absolute paths via `/**` glob.
fn parse_worktree_args(args: &[String]) -> Result<ParsedCommand, Report<CommandParseError>> {
    let schema = CommandSchemaBuilder::new("git")
        .with_subcommand(
            CommandSchemaBuilder::new("worktree")
                .with_subcommand(
                    CommandSchemaBuilder::new("add")
                        .with_option(
                            OptionSchemaBuilder::new(["-b"])
                                .with_value(ValueConstraint::Any)
                                .build(),
                        )
                        .with_operand(
                            OperandSchemaBuilder::new("path")
                                .with_value(
                                    ValueConstraint::glob("/**").expect("/** is a valid glob"),
                                )
                                .build(),
                        )
                        .with_operand(
                            OperandSchemaBuilder::new("commit-ish")
                                .with_optional()
                                .build(),
                        )
                        .build(),
                )
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
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result =
            eval_rules_with_settings(git_worktree_rules(), "git worktree add /a/wt/foo", settings);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_with_b() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo -b feat",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_b_before_path() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add -b feat /a/wt/foo",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_with_commit() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo main",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_quoted() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add \"/a/wt/foo\" -b feat",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_untrusted() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /tmp/evil -b feat",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_negated() {
        let settings = Settings::with_worktrees(&["/a/**", "!/a/blocked/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/blocked/foo -b feat",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Branch name matches trusted pattern but actual path is untrusted.
    #[test]
    fn trusted_path_b_value_resembles_path() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add -b /a/wt/decoy /tmp/evil",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_no_patterns() {
        let settings = Settings::with_worktrees(&[]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo -b feat",
            settings,
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unsupported_flags_detach() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo --detach",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_force() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo --force",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_capital_b() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo -B my-branch",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_orphan() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add /a/wt/foo --orphan",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_lock() {
        let settings = Settings::with_worktrees(&["/a/wt/**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add --lock /a/wt/foo",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn trusted_path_mock_settings() {
        let result = eval_rules(
            git_worktree_rules(),
            "git worktree add /home/user/worktrees/my-project -b feat",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Relative path is rejected by the operand schema.
    #[test]
    fn trusted_path_relative() {
        let settings = Settings::with_worktrees(&["./**"]);
        let result = eval_rules_with_settings(
            git_worktree_rules(),
            "git worktree add ./worktree -b feat",
            settings,
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
