//! Deny traditional tools and suggest modern alternatives.

use crate::prelude::*;

/// Deny `find`, `grep`, and `sed` in favor of `fd`, `rg`, and `sd`.
pub fn modern_alternative_rules() -> Vec<BashRule> {
    vec![find(), grep(), sed()]
}

/// Deny `find` in favor of `fd`.
fn find() -> BashRule {
    BashRule::new(
        "find",
        "find",
        Outcome::deny("find is blocked. Use 'fd' instead (e.g., 'fd -e rs' to find *.rs files)."),
    )
}

/// Deny `grep` in favor of `rg`.
fn grep() -> BashRule {
    BashRule::new(
        "grep",
        "grep",
        Outcome::deny("grep is blocked. Use 'rg' instead (e.g., 'rg pattern' or 'rg -l pattern')."),
    )
}

/// Deny `sed` in favor of `sd`.
fn sed() -> BashRule {
    BashRule::new(
        "sed",
        "sed",
        Outcome::deny("sed is blocked. Use 'sd' instead (e.g., 'sd before after file.txt')."),
    )
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // === find denied ===

    #[test]
    fn find_standalone() {
        let outcome = evaluate_expect_outcome("find . -name '*.rs'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_piped() {
        let outcome = evaluate_expect_outcome("find . -name '*.rs' | head -5");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_chained() {
        let outcome = evaluate_expect_outcome("echo start && find . -type f");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    // === grep denied ===

    #[test]
    fn grep_standalone() {
        let outcome = evaluate_expect_outcome("grep pattern file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn grep_piped() {
        let outcome = evaluate_expect_outcome("cat file.txt | grep pattern");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn grep_chained() {
        let outcome = evaluate_expect_outcome("echo start && grep -r pattern .");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    // === sed denied ===

    #[test]
    fn sed_standalone() {
        let outcome = evaluate_expect_outcome("sed 's/old/new/' file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sed_piped() {
        let outcome = evaluate_expect_outcome("cat file.txt | sed 's/old/new/'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sed_chained() {
        let outcome = evaluate_expect_outcome("echo start && sed 's/old/new/' file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    // === modern tools pass through ===

    #[test]
    fn fd_allowed() {
        let outcome = evaluate_expect_outcome("fd -e rs");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_allowed() {
        let outcome = evaluate_expect_outcome("rg pattern file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn sd_passthrough() {
        let reason = evaluate_expect_skip("sd before after file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    // === echo not affected ===

    #[test]
    fn echo_with_find() {
        let outcome = evaluate_expect_outcome("echo 'use find to search'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_with_grep() {
        let outcome = evaluate_expect_outcome("echo 'use grep to search'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_with_sed() {
        let outcome = evaluate_expect_outcome("echo 'use sed to replace'");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
