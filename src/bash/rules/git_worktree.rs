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

/// True if any flag other than `-b` is present in the args after `worktree add`.
fn has_unsupported_flags(
    context: &SimpleContext,
    _complete: &CompleteContext,
    _settings: &Settings,
) -> bool {
    let remaining = match context.args.get(2..) {
        Some(rest) if !rest.is_empty() => rest,
        _ => return false,
    };
    parse_worktree_args(remaining).is_err()
}

/// True if the first positional `/`-prefixed argument matches a trusted worktree path.
///
/// Uses [`ArgParser`] to correctly skip flag values. Git rejects refs
/// starting with `/` so this is defense-in-depth rather than a
/// practical bypass today.
fn is_worktree_path_trusted(
    context: &SimpleContext,
    _complete: &CompleteContext,
    settings: &Settings,
) -> bool {
    let remaining = match context.args.get(2..) {
        Some(rest) if !rest.is_empty() => rest,
        _ => return false,
    };
    let Ok(parsed) = parse_worktree_args(remaining) else {
        return false;
    };
    let factory = PathRuleFactory::default();
    for arg in &parsed {
        if let Arg::Operand(value) = arg {
            if !value.starts_with('/') {
                continue;
            }
            if let Some(is_allowed) = factory.is_match(value, &settings.worktrees.paths) {
                trace!(is_allowed, "Matched worktree path");
                return is_allowed;
            }
        }
    }
    trace!("No worktree path match");
    false
}

/// Parse args after `worktree add` using a schema that only allows `-b`.
fn parse_worktree_args(args: &[String]) -> Result<Vec<Arg>, Report<ArgParseError>> {
    let settings = ArgParserSettings {
        schema: ArgSchema {
            bool_flags: vec![],
            value_flags: vec![String::from("-b")],
        },
        unquote: true,
    };
    ArgParser::new(settings).parse(args.to_vec())
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn trusted_path() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_with_b() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo -b feat", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_b_before_path() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add -b feat /a/wt/foo", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_with_commit() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo main", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_quoted() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add \"/a/wt/foo\" -b feat", settings);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn trusted_path_untrusted() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let reason = eval_skip("git worktree add /tmp/evil -b feat", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_negated() {
        let settings = worktree_settings(&["/a/**", "!/a/blocked/**"]);
        let reason = eval_skip("git worktree add /a/blocked/foo -b feat", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Branch name matches trusted pattern but actual path is untrusted.
    #[test]
    fn trusted_path_b_value_resembles_path() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let reason = eval_skip("git worktree add -b /a/wt/decoy /tmp/evil", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn trusted_path_no_patterns() {
        let settings = worktree_settings(&[]);
        let reason = eval_skip("git worktree add /a/wt/foo -b feat", settings);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn unsupported_flags_detach() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo --detach", settings);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_force() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo --force", settings);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_capital_b() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo -B my-branch", settings);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_orphan() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add /a/wt/foo --orphan", settings);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn unsupported_flags_lock() {
        let settings = worktree_settings(&["/a/wt/**"]);
        let outcome = eval_outcome("git worktree add --lock /a/wt/foo", settings);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn trusted_path_mock_settings() {
        let outcome =
            evaluate_expect_outcome("git worktree add /home/user/worktrees/my-project -b feat");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    fn worktree_settings(paths: &[&str]) -> Settings {
        Settings {
            worktrees: WorktreeSettings {
                paths: paths.iter().map(|s| String::from(*s)).collect(),
            },
            ..Default::default()
        }
    }

    fn eval_outcome(command: &str, settings: Settings) -> Outcome {
        let _logger = init_test_logger();
        BashEvaluator::new(settings)
            .evaluate_str(command)
            .expect("command should produce an outcome")
    }

    #[expect(clippy::panic, reason = "test helper")]
    fn eval_skip(command: &str, settings: Settings) -> SkipReason {
        let _logger = init_test_logger();
        match BashEvaluator::new(settings)
            .evaluate_str(command)
            .expect_err("command should not succeed")
            .current_context()
        {
            ParseError::Skip(reason) => *reason,
            other => panic!("expected Skip, got {other:?}"),
        }
    }
}
