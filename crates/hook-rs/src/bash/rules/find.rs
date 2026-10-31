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
            ArgMatcher::new("-delete"),
            ArgMatcher::new("-exec"),
            ArgMatcher::new("-execdir"),
            ArgMatcher::new("-ok"),
            ArgMatcher::new("-okdir"),
            ArgMatcher::new("-fprint"),
            ArgMatcher::new("-fprint0"),
            ArgMatcher::new("-fprintf"),
            ArgMatcher::new("-fls"),
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
        with_any: Some(vec![ArgMatcher::new("-delete")]),
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
            ArgMatcher::new("-exec").value("rm"),
            ArgMatcher::new("-execdir").value("rm"),
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
        let result = eval_rules(find_rules(), "find . -name '*.tmp' -delete");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_path() {
        let result = eval_rules(find_rules(), "find /path -type f -delete");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_redirect() {
        let result = eval_rules(find_rules(), "find . -name .lock -delete 2>/dev/null");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_exec_rm() {
        let result = eval_rules(find_rules(), "find . -name '*.tmp' -exec rm {} \\;");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_exec_rm_f() {
        let result = eval_rules(find_rules(), "find . -type f -exec rm -f {} +");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_exec_rm_execdir() {
        let result = eval_rules(find_rules(), "find . -name '*.log' -execdir rm {} \\;");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_chained() {
        let result = eval_rules(find_rules(), "ls && find . -delete");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_semicolon() {
        let result = eval_rules(find_rules(), "echo test ; find . -name '*.tmp' -delete");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn find_name() {
        let result = eval_rules(find_rules(), "find . -name '*.rs'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn find_print() {
        let result = eval_rules(find_rules(), "find . -type f -print");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn find_maxdepth() {
        let result = eval_rules(find_rules(), "find /path -maxdepth 1");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn find_exec_ls() {
        let result = eval_rules(find_rules(), "find . -name '*.tmp' -exec ls {} \\;");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn find_exec_cat() {
        let result = eval_rules(find_rules(), "find . -name '*.txt' -exec cat {} +");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_find_delete() {
        let result = eval_rules(find_rules(), "echo 'find -delete is dangerous'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
