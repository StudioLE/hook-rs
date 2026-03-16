//! Deny rules for destructive `find` operations.

use crate::prelude::*;

/// Deny `find -delete` and `find -exec rm` to prevent bulk file deletion.
pub fn find_rules() -> Vec<BashRule> {
    vec![find_delete(), find_exec_rm()]
}

/// Deny `find -delete`.
fn find_delete() -> BashRule {
    BashRule {
        id: "find_delete".to_owned(),
        command: "find".to_owned(),
        with_any: Some(vec![Arg::new("-delete")]),
        outcome: Outcome::deny(
            "find -delete is blocked. Use 'find ... -print' to preview matches first, \
             then delete with targeted commands.",
        ),
        ..Default::default()
    }
}

/// Deny `find -exec rm`.
fn find_exec_rm() -> BashRule {
    BashRule {
        id: "find_exec_rm".to_owned(),
        command: "find".to_owned(),
        with_any: Some(vec![
            Arg::new("-exec").value("rm"),
            Arg::new("-execdir").value("rm"),
        ]),
        outcome: Outcome::deny(
            "find -exec rm is blocked. Use 'find ... -print' to preview matches first, \
             then delete with targeted commands.",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn _find_delete() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -delete");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_delete__path() {
        let outcome = evaluate_expect_outcome("find /path -type f -delete");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_delete__redirect() {
        let outcome = evaluate_expect_outcome("find . -name .lock -delete 2>/dev/null");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_exec_rm() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -exec rm {} \\;");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_exec_rm__f() {
        let outcome = evaluate_expect_outcome("find . -type f -exec rm -f {} +");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_exec_rm__execdir() {
        let outcome = evaluate_expect_outcome("find . -name '*.log' -execdir rm {} \\;");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_delete__chained() {
        let outcome = evaluate_expect_outcome("ls && find . -delete");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_delete__semicolon() {
        let outcome = evaluate_expect_outcome("echo test ; find . -name '*.tmp' -delete");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _find_name() {
        let reason = evaluate_expect_skip("find . -name '*.rs'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_print() {
        let reason = evaluate_expect_skip("find . -type f -print");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_maxdepth() {
        let reason = evaluate_expect_skip("find /path -maxdepth 1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_exec_ls() {
        let reason = evaluate_expect_skip("find . -name '*.tmp' -exec ls {} \\;");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_exec_cat() {
        let reason = evaluate_expect_skip("find . -name '*.txt' -exec cat {} +");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _echo_find_delete() {
        let outcome = evaluate_expect_outcome("echo 'find -delete is dangerous'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _grep_delete() {
        let outcome = evaluate_expect_outcome("grep -r 'delete' .");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_log_grep_delete() {
        // git log is Allow via git_approval, grep is Allow via safe_rules
        let outcome = evaluate_expect_outcome("git log --oneline | grep delete");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
