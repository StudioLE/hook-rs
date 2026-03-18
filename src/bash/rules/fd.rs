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
            Arg::new("-x"),
            Arg::new("--exec"),
            Arg::new("-X"),
            Arg::new("--exec-batch"),
        ]),
        outcome: Outcome::allow("Safe command: fd (no exec)"),
        ..Default::default()
    }
}

/// Deny `fd -x rm` and `fd --exec rm` variants.
fn fd_exec_rm() -> BashRule {
    BashRule {
        id: "fd_exec_rm".to_owned(),
        command: "fd".to_owned(),
        with_any: Some(vec![
            Arg::new("-x").value("rm"),
            Arg::new("--exec").value("rm"),
            Arg::new("-X").value("rm"),
            Arg::new("--exec-batch").value("rm"),
        ]),
        outcome: Outcome::deny(
            "fd -x rm is blocked. Use 'fd ... --list-details' to preview matches first, \
             then delete with targeted commands.",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn _fd_exec_rm() {
        let outcome = evaluate_expect_outcome("fd -e tmp -x rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _fd_exec_rm__long() {
        let outcome = evaluate_expect_outcome("fd -e tmp --exec rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _fd_exec_batch_rm() {
        let outcome = evaluate_expect_outcome("fd -e tmp -X rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _fd_exec_batch_rm__long() {
        let outcome = evaluate_expect_outcome("fd -e tmp --exec-batch rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _fd_exec_rm__chained() {
        let outcome = evaluate_expect_outcome("ls && fd -e tmp -x rm");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _fd_read_only() {
        let outcome = evaluate_expect_outcome("fd -e rs");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _fd_read_only__pattern() {
        let outcome = evaluate_expect_outcome("fd 'test.*' src/");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _fd_read_only__piped() {
        let outcome = evaluate_expect_outcome("fd -e rs | head -20");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _fd_exec_ls() {
        let reason = evaluate_expect_skip("fd -e rs -x ls");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _fd_exec_cat() {
        let reason = evaluate_expect_skip("fd -e txt --exec cat");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _echo_fd() {
        let outcome = evaluate_expect_outcome("echo 'fd -x rm is dangerous'");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
