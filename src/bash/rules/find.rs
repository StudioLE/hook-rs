//! Rules for `find` operations: allow read-only, deny destructive.

use crate::prelude::*;

/// Rules for `find`.
pub fn find_rules() -> Vec<BashRule> {
    vec![find_delete(), find_exec_rm(), find__read_only()]
}

/// Allow `find` without exec flags.
fn find__read_only() -> BashRule {
    BashRule {
        id: "find__read_only".to_owned(),
        command: "find".to_owned(),
        without_any: Some(vec![
            Arg::new("-delete"),
            Arg::new("-exec"),
            Arg::new("-execdir"),
            Arg::new("-ok"),
            Arg::new("-okdir"),
            Arg::new("-fprint"),
            Arg::new("-fprint0"),
            Arg::new("-fprintf"),
            Arg::new("-fls"),
        ]),
        outcome: Outcome::allow("Read-only `find`"),
        ..Default::default()
    }
}

/// Deny `find -delete`.
fn find_delete() -> BashRule {
    BashRule {
        id: "find_delete".to_owned(),
        command: "find".to_owned(),
        with_any: Some(vec![Arg::new("-delete")]),
        outcome: Outcome::deny(
            "`find -delete` is blocked. Alternatives: `find ... -print` to preview, \
             then `git rm` / `git clean`",
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
            "`find -exec rm` is blocked. Alternatives: `find ... -print` to preview, \
             then `git rm` / `git clean`",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn find_delete() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_path() {
        let outcome = evaluate_expect_outcome("find /path -type f -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_redirect() {
        let outcome = evaluate_expect_outcome("find . -name .lock -delete 2>/dev/null");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_exec_rm() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -exec rm {} \\;");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_exec_rm_f() {
        let outcome = evaluate_expect_outcome("find . -type f -exec rm -f {} +");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_exec_rm_execdir() {
        let outcome = evaluate_expect_outcome("find . -name '*.log' -execdir rm {} \\;");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_chained() {
        let outcome = evaluate_expect_outcome("ls && find . -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_semicolon() {
        let outcome = evaluate_expect_outcome("echo test ; find . -name '*.tmp' -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_name() {
        let outcome = evaluate_expect_outcome("find . -name '*.rs'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn find_print() {
        let outcome = evaluate_expect_outcome("find . -type f -print");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn find_maxdepth() {
        let outcome = evaluate_expect_outcome("find /path -maxdepth 1");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn find_exec_ls() {
        let reason = evaluate_expect_skip("find . -name '*.tmp' -exec ls {} \\;");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn find_exec_cat() {
        let reason = evaluate_expect_skip("find . -name '*.txt' -exec cat {} +");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_find_delete() {
        let outcome = evaluate_expect_outcome("echo 'find -delete is dangerous'");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
