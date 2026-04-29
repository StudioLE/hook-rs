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
        let outcome = evaluate_expect_outcome("rm -r /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_cap_r() {
        let outcome = evaluate_expect_outcome("rm -R /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf() {
        let outcome = evaluate_expect_outcome("rm -rf /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_cap_rf() {
        let outcome = evaluate_expect_outcome("rm -Rf /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_fr() {
        let outcome = evaluate_expect_outcome("rm -fr /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f_cap_r() {
        let outcome = evaluate_expect_outcome("rm -fR /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_recursive() {
        let outcome = evaluate_expect_outcome("rm --recursive /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rfi() {
        let outcome = evaluate_expect_outcome("rm -rfi /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_ir() {
        let outcome = evaluate_expect_outcome("rm -ir /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_single_file() {
        let outcome = evaluate_expect_outcome("rm file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_multiple_files() {
        let outcome = evaluate_expect_outcome("rm file1.txt file2.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f() {
        let outcome = evaluate_expect_outcome("rm -f file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_i() {
        let outcome = evaluate_expect_outcome("rm -i file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_with_path() {
        let outcome = evaluate_expect_outcome("rm /path/to/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_wildcard() {
        let outcome = evaluate_expect_outcome("rm *.tmp");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_r_chained() {
        let outcome = evaluate_expect_outcome("ls && rm -r /path");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf_or_chain() {
        let outcome = evaluate_expect_outcome("false || rm -rf /tmp/nothing");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_r_semicolon() {
        let outcome = evaluate_expect_outcome("echo hi ; rm -r /path");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_chained_file() {
        let outcome = evaluate_expect_outcome("ls && rm file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_for_do() {
        let outcome = evaluate_expect_outcome("for f in *.tmp; do rm $f; done");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_if_then() {
        let reason = evaluate_expect_skip("if true; then rm file.txt; fi");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn rm_if_else() {
        let reason = evaluate_expect_skip("if false; then echo hi; else rm file.txt; fi");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn rm_rf_while_do() {
        let reason = evaluate_expect_skip("while true; do rm -rf /tmp/nothing; done");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }
    #[test]
    fn rm_tmp_file() {
        let outcome = evaluate_expect_outcome("rm /tmp/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f_tmp() {
        let outcome = evaluate_expect_outcome("rm -f /tmp/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf_tmp() {
        let outcome = evaluate_expect_outcome("rm -rf /tmp/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_multiple() {
        let outcome = evaluate_expect_outcome("rm /tmp/file1 /tmp/file2");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_path_traversal() {
        let outcome = evaluate_expect_outcome("rm /tmp/../etc/passwd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_mixed() {
        let outcome = evaluate_expect_outcome("rm /tmp/file.txt /home/user/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn ls() {
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_rm() {
        let outcome = evaluate_expect_outcome("git rm file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_rm_r() {
        let outcome = evaluate_expect_outcome("git rm -r dir/");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_rm() {
        let outcome = evaluate_expect_outcome("echo rm is blocked");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_rm() {
        let outcome = evaluate_expect_outcome("rg rm .");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cat() {
        let outcome = evaluate_expect_outcome("cat file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn mv() {
        let reason = evaluate_expect_skip("mv old.txt new.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_rm() {
        let reason = evaluate_expect_skip("cargo rm some-dep");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rm_snap_new() {
        let outcome = evaluate_expect_outcome("rm path/to/foo.snap.new");
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_dot_suffix() {
        let outcome = evaluate_expect_outcome("rm path/to/foo.snap.new.42");
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_glob() {
        let outcome = evaluate_expect_outcome("rm crates/core/snapshots/foo__bar.snap.new");
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_mixed_with_other() {
        let outcome = evaluate_expect_outcome("rm foo.snap.new other.txt");
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
        assert!(!outcome.reason.contains("git clean"));
    }

    #[test]
    fn rm_pending_snap_inline() {
        let outcome = evaluate_expect_outcome("rm src/.foo.rs.pending-snap");
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_not_new() {
        let outcome = evaluate_expect_outcome("rm foo.snap");
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("git rm"));
    }

    #[test]
    fn xargs_rm() {
        let reason = evaluate_expect_skip("echo file | xargs rm");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }
}
