//! Rules for `fd` operations: allow read-only, deny destructive.

use crate::prelude::*;

/// Deny destructive `fd`, allow read-only `fd`.
pub fn fd_rules() -> Vec<BashRule> {
    vec![fd_exec_rm(), fd__read_only()]
}

/// Allow `fd` without exec flags.
fn fd__read_only() -> BashRule {
    BashRule {
        id: "fd__read_only".to_owned(),
        command: "fd".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("-x"),
            ArgMatcher::new("--exec"),
            ArgMatcher::new("-X"),
            ArgMatcher::new("--exec-batch"),
        ]),
        outcome: Outcome::allow("Read-only `fd`"),
        ..Default::default()
    }
}

/// Deny `fd -x rm` and `fd --exec rm` variants.
fn fd_exec_rm() -> BashRule {
    BashRule {
        id: "fd_exec_rm".to_owned(),
        command: "fd".to_owned(),
        with_any: Some(vec![
            ArgMatcher::new("-x").value("rm"),
            ArgMatcher::new("--exec").value("rm"),
            ArgMatcher::new("-X").value("rm"),
            ArgMatcher::new("--exec-batch").value("rm"),
        ]),
        outcome: Outcome::deny(
            "`fd -x rm` is blocked. Alternatives: `fd ... --list-details` to preview, \
             then `git rm` / `git clean`",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn fd_exec_rm() {
        let outcome = evaluate_expect_outcome("fd -e tmp -x rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn fd_exec_rm_long() {
        let outcome = evaluate_expect_outcome("fd -e tmp --exec rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn fd_exec_batch_rm() {
        let outcome = evaluate_expect_outcome("fd -e tmp -X rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn fd_exec_batch_rm_long() {
        let outcome = evaluate_expect_outcome("fd -e tmp --exec-batch rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn fd_exec_rm_chained() {
        let outcome = evaluate_expect_outcome("ls && fd -e tmp -x rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn fd_read_only() {
        let outcome = evaluate_expect_outcome("fd -e rs");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn fd_read_only_pattern() {
        let outcome = evaluate_expect_outcome("fd 'test.*' src/");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn fd_read_only_piped() {
        let outcome = evaluate_expect_outcome("fd -e rs | head -20");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn fd_exec_ls() {
        let reason = evaluate_expect_skip("fd -e rs -x ls");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn fd_exec_cat() {
        let reason = evaluate_expect_skip("fd -e txt --exec cat");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_fd() {
        let outcome = evaluate_expect_outcome("echo 'fd -x rm is dangerous'");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
