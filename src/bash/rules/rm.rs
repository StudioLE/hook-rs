//! Deny rules for `rm` to prevent file deletion.

use crate::prelude::*;

/// Deny `rm` invocations.
#[must_use]
pub fn rm_rules() -> Vec<BashRule> {
    vec![rm(), rm__snap_new()]
}

/// Deny all `rm` invocations, directing to `git rm` or `git clean` instead.
fn rm() -> BashRule {
    BashRule {
        id: "rm".to_owned(),
        command: "rm".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("**/*.snap.new"),
            ArgMatcher::new("**/*.snap.new.*"),
            ArgMatcher::new("**/.*.pending-snap"),
        ]),
        outcome: Outcome::deny(
            "`rm` is blocked. Alternatives: `git rm -f <file>`, `git rm -fx <file>` (gitignored), \
             `git clean -f <file>`, `git clean -fx <file>` (gitignored)",
        ),
        ..Default::default()
    }
}

/// Deny `rm` of pending insta snapshots, directing to `cargo insta` workflow.
fn rm__snap_new() -> BashRule {
    BashRule {
        id: "rm__snap_new".to_owned(),
        command: "rm".to_owned(),
        with_any: Some(vec![
            ArgMatcher::new("**/*.snap.new"),
            ArgMatcher::new("**/*.snap.new.*"),
            ArgMatcher::new("**/.*.pending-snap"),
        ]),
        outcome: Outcome::deny(
            "`rm` of pending insta snapshots is blocked. Use `cargo insta accept` to accept or \
             `cargo insta reject` to reject pending snapshots",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn rm_r() {
        let result = eval_rules(rm_rules(), "rm -r /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_cap_r() {
        let result = eval_rules(rm_rules(), "rm -R /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf() {
        let result = eval_rules(rm_rules(), "rm -rf /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_cap_rf() {
        let result = eval_rules(rm_rules(), "rm -Rf /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_fr() {
        let result = eval_rules(rm_rules(), "rm -fr /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f_cap_r() {
        let result = eval_rules(rm_rules(), "rm -fR /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_recursive() {
        let result = eval_rules(rm_rules(), "rm --recursive /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rfi() {
        let result = eval_rules(rm_rules(), "rm -rfi /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_ir() {
        let result = eval_rules(rm_rules(), "rm -ir /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_single_file() {
        let result = eval_rules(rm_rules(), "rm file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_multiple_files() {
        let result = eval_rules(rm_rules(), "rm file1.txt file2.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f() {
        let result = eval_rules(rm_rules(), "rm -f file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_i() {
        let result = eval_rules(rm_rules(), "rm -i file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_with_path() {
        let result = eval_rules(rm_rules(), "rm /path/to/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_wildcard() {
        let result = eval_rules(rm_rules(), "rm *.tmp");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_r_chained() {
        let result = eval_rules(rm_rules(), "ls && rm -r /path");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf_or_chain() {
        let result = eval_rules(rm_rules(), "false || rm -rf /tmp/nothing");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_r_semicolon() {
        let result = eval_rules(rm_rules(), "echo hi ; rm -r /path");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_chained_file() {
        let result = eval_rules(rm_rules(), "ls && rm file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_for_do() {
        let result = eval_rules(rm_rules(), "for f in *.tmp; do rm $f; done");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_if_then() {
        let result = eval_rules(rm_rules(), "if true; then rm file.txt; fi");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn rm_if_else() {
        let result = eval_rules(rm_rules(), "if false; then echo hi; else rm file.txt; fi");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn rm_rf_while_do() {
        let result = eval_rules(rm_rules(), "while true; do rm -rf /tmp/nothing; done");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }
    #[test]
    fn rm_tmp_file() {
        let result = eval_rules(rm_rules(), "rm /tmp/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f_tmp() {
        let result = eval_rules(rm_rules(), "rm -f /tmp/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf_tmp() {
        let result = eval_rules(rm_rules(), "rm -rf /tmp/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_multiple() {
        let result = eval_rules(rm_rules(), "rm /tmp/file1 /tmp/file2");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_path_traversal() {
        let result = eval_rules(rm_rules(), "rm /tmp/../etc/passwd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_mixed() {
        let result = eval_rules(rm_rules(), "rm /tmp/file.txt /home/user/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn ls() {
        let result = eval_rules(rm_rules(), "ls -la");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_rm() {
        let result = eval_rules(rm_rules(), "git rm file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_rm_r() {
        let result = eval_rules(rm_rules(), "git rm -r dir/");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_rm() {
        let result = eval_rules(rm_rules(), "echo rm is blocked");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_rm() {
        let result = eval_rules(rm_rules(), "rg rm .");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cat() {
        let result = eval_rules(rm_rules(), "cat file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mv() {
        let result = eval_rules(rm_rules(), "mv old.txt new.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_rm() {
        let result = eval_rules(rm_rules(), "cargo rm some-dep");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rm_snap_new() {
        let result = eval_rules(rm_rules(), "rm path/to/foo.snap.new");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_dot_suffix() {
        let result = eval_rules(rm_rules(), "rm path/to/foo.snap.new.42");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_glob() {
        let result = eval_rules(rm_rules(), "rm crates/core/snapshots/foo__bar.snap.new");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_mixed_with_other() {
        let result = eval_rules(rm_rules(), "rm foo.snap.new other.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
        assert!(!outcome.reason.contains("git clean"));
    }

    #[test]
    fn rm_pending_snap_inline() {
        let result = eval_rules(rm_rules(), "rm src/.foo.rs.pending-snap");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_not_new() {
        let result = eval_rules(rm_rules(), "rm foo.snap");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("git rm"));
    }

    #[test]
    fn xargs_rm() {
        let result = eval_rules(rm_rules(), "echo file | xargs rm");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
