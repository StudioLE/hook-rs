//! Rules for `rg` operations: allow read-only, deny short `-r` replacement and ignored files.

use crate::prelude::*;
use std::ptr;

/// Deny `rg -r` and `rg` without `--no-ignore`, allow read-only `rg`.
pub fn rg_rules() -> Vec<BashRule> {
    vec![rg__replace(), rg__ignored(), rg__read_only()]
}

/// Deny `rg` searching files without `--no-ignore`.
///
/// - By default `rg` skips gitignored files so missing results don't prove absence
/// - Skips `rg` reading piped stdin since ignore rules only apply to files
fn rg__ignored() -> BashRule {
    BashRule {
        id: "rg__ignored".to_owned(),
        command: "rg".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("--no-ignore"),
            ArgMatcher::new("-u"),
            ArgMatcher::new("--unrestricted"),
        ]),
        condition: Some(|ctx| !is_piped_into(ctx)),
        outcome: Outcome::deny(
            "`rg` skips gitignored and hidden files by default, so missing results don't prove a file is absent. Add `--no-ignore --hidden`",
        ),
        ..Default::default()
    }
}

/// Is a preceding command at the same nesting level piped into this command?
fn is_piped_into(ctx: &BashRuleContext) -> bool {
    let is_current = |simple: &SimpleContext| ptr::eq(simple, ctx.simple);
    let Some(pipeline) = ctx
        .complete
        .children
        .iter()
        .find(|pipeline| pipeline.children.iter().any(is_current))
    else {
        return false;
    };
    pipeline
        .children
        .iter()
        .take_while(|simple| !is_current(simple))
        .any(|simple| simple.nesting == ctx.simple.nesting)
}

/// Allow `rg` without a short `-r`.
fn rg__read_only() -> BashRule {
    BashRule {
        id: "rg__read_only".to_owned(),
        command: "rg".to_owned(),
        without_any: Some(vec![ArgMatcher::new("-r")]),
        outcome: Outcome::allow("Read-only `rg`"),
        ..Default::default()
    }
}

/// Deny short `rg -r`; point at `rg -n` and the long `--replace` form.
///
/// `-r` is `--replace` in `rg`, not `--recursive`, so `-rn` silently replaces
/// every match with the literal `n`.
fn rg__replace() -> BashRule {
    BashRule {
        id: "rg__replace".to_owned(),
        command: "rg".to_owned(),
        with_any: Some(vec![ArgMatcher::new("-r")]),
        outcome: Outcome::deny(
            "`rg -r` is `--replace`, not `--recursive`, so it rewrites matches in the output. `rg` already recurses. Use `rg -n`, or `rg --replace=TEXT` to replace deliberately",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn rg_replace_concatenated() {
        let result = eval_rules(rg_rules(), "rg -rn 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_replace_separate() {
        let result = eval_rules(rg_rules(), "rg -r n 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_replace_bundled() {
        let result = eval_rules(rg_rules(), "rg -inr 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_replace_long() {
        let result = eval_rules(
            rg_rules(),
            "rg --no-ignore --replace='$1' 'v(\\d+)' file.txt",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_line_numbers() {
        let result = eval_rules(rg_rules(), "rg --no-ignore -n 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_ignored() {
        let result = eval_rules(rg_rules(), "rg -n 'fn main' src/");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_ignored_hidden_only() {
        let result = eval_rules(rg_rules(), "rg --hidden 'fn main'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    /// `--no-ignore-vcs` still respects `.ignore` and `.rgignore`.
    #[test]
    fn rg_ignored_partial() {
        let result = eval_rules(rg_rules(), "rg --no-ignore-vcs 'fn main'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_ignored_chained() {
        let result = eval_rules(rg_rules(), "ls && rg 'fn main'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_no_ignore_hidden() {
        let result = eval_rules(rg_rules(), "rg --no-ignore --hidden 'fn main'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_unrestricted_bundled() {
        let result = eval_rules(rg_rules(), "rg -uun 'fn main'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_unrestricted_long() {
        let result = eval_rules(rg_rules(), "rg --unrestricted 'fn main'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_stdin() {
        let result = eval_rules(rg_rules(), "cargo test 2>&1 | rg FAIL");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn rg_stdin_substitution() {
        let result = eval_rules(rg_rules(), "echo $(git log | rg fix)");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    /// `rg` inside a substitution is not piped into by the outer command.
    #[test]
    fn rg_ignored_substitution() {
        let result = eval_rules(rg_rules(), "echo $(rg -l 'fn main')");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_ignored_piped_out() {
        let result = eval_rules(rg_rules(), "rg 'fn main' | head");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
