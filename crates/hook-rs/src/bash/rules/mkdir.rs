//! Allow rules for `mkdir` without setting modes or security contexts.

use crate::prelude::*;

/// Rules for `mkdir`.
#[must_use]
pub fn mkdir_rules() -> Vec<BashRule> {
    vec![mkdir()]
}

/// Allow `mkdir` without `-m`/`--mode` or `-Z`/`--context`.
///
/// - `-m` sets permission bits such as `777` or setgid
/// - `-Z` and `--context` set the `SELinux` security context
/// - `--m*` and `--c*` also catch GNU abbreviations such as `--mo`
fn mkdir() -> BashRule {
    BashRule {
        id: "mkdir".to_owned(),
        command: "mkdir".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("-m"),
            ArgMatcher::new("--m*"),
            ArgMatcher::new("-Z"),
            ArgMatcher::new("--c*"),
        ]),
        outcome: Outcome::allow("Create directories with `mkdir`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn mkdir_path() {
        let result = eval_rules(mkdir_rules(), "mkdir build");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn mkdir_parents_verbose() {
        let result = eval_rules(mkdir_rules(), "mkdir -p -v a/b/c d");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn mkdir_parents_verbose_bundled() {
        let result = eval_rules(mkdir_rules(), "mkdir -pv a/b/c");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn mkdir_parents_verbose_long() {
        let result = eval_rules(mkdir_rules(), "mkdir --parents --verbose a/b/c");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn mkdir_mode() {
        let result = eval_rules(mkdir_rules(), "mkdir -m 777 shared");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mkdir_mode_bundled() {
        let result = eval_rules(mkdir_rules(), "mkdir -pm777 a/shared");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mkdir_mode_long() {
        let result = eval_rules(mkdir_rules(), "mkdir --mode=2775 shared");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// GNU accepts `--m` as an abbreviation of `--mode`.
    #[test]
    fn mkdir_mode_abbreviated() {
        let result = eval_rules(mkdir_rules(), "mkdir --m=777 shared");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mkdir_context() {
        let result = eval_rules(mkdir_rules(), "mkdir -Z labelled");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mkdir_context_bundled() {
        let result = eval_rules(mkdir_rules(), "mkdir -pZ a/labelled");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mkdir_context_long() {
        let result = eval_rules(
            mkdir_rules(),
            "mkdir --context=system_u:object_r:shadow_t:s0 labelled",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// GNU accepts `--c` as an abbreviation of `--context`.
    #[test]
    fn mkdir_context_abbreviated() {
        let result = eval_rules(mkdir_rules(), "mkdir --c labelled");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
