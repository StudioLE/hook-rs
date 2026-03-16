//! Deny rule for `rm` to prevent file deletion.

use crate::prelude::*;

/// Deny all `rm` invocations, directing to `git rm` or `git clean` instead.
#[must_use]
pub fn rm() -> BashRule {
    BashRule::new(
        "rm",
        "rm",
        Outcome::deny(
            "rm is blocked. Use 'git rm -f <file>' for specific files (or -fx if gitignored) \
             or 'git clean -f <file>' for untracked files (or -fx if gitignored).",
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn _rm_r() {
        let outcome = evaluate_expect_outcome("rm -r /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_cap_r() {
        let outcome = evaluate_expect_outcome("rm -R /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_rf() {
        let outcome = evaluate_expect_outcome("rm -rf /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_cap_rf() {
        let outcome = evaluate_expect_outcome("rm -Rf /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_fr() {
        let outcome = evaluate_expect_outcome("rm -fr /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_f_cap_r() {
        let outcome = evaluate_expect_outcome("rm -fR /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_recursive() {
        let outcome = evaluate_expect_outcome("rm --recursive /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_rfi() {
        let outcome = evaluate_expect_outcome("rm -rfi /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_ir() {
        let outcome = evaluate_expect_outcome("rm -ir /path/to/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_single_file() {
        let outcome = evaluate_expect_outcome("rm file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_multiple_files() {
        let outcome = evaluate_expect_outcome("rm file1.txt file2.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_f() {
        let outcome = evaluate_expect_outcome("rm -f file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_i() {
        let outcome = evaluate_expect_outcome("rm -i file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_with_path() {
        let outcome = evaluate_expect_outcome("rm /path/to/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_wildcard() {
        let outcome = evaluate_expect_outcome("rm *.tmp");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_r__chained() {
        let outcome = evaluate_expect_outcome("ls && rm -r /path");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_rf__or_chain() {
        let outcome = evaluate_expect_outcome("false || rm -rf /tmp/nothing");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_r__semicolon() {
        let outcome = evaluate_expect_outcome("echo hi ; rm -r /path");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm__chained_file() {
        let outcome = evaluate_expect_outcome("ls && rm file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm__for_do() {
        let outcome = evaluate_expect_outcome("for f in *.tmp; do rm $f; done");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm__if_then() {
        let reason = evaluate_expect_skip("if true; then rm file.txt; fi");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn _rm__if_else() {
        let reason = evaluate_expect_skip("if false; then echo hi; else rm file.txt; fi");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn _rm_rf__while_do() {
        let reason = evaluate_expect_skip("while true; do rm -rf /tmp/nothing; done");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }
    #[test]
    fn _rm__tmp_file() {
        let outcome = evaluate_expect_outcome("rm /tmp/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_f__tmp() {
        let outcome = evaluate_expect_outcome("rm -f /tmp/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm_rf__tmp() {
        let outcome = evaluate_expect_outcome("rm -rf /tmp/dir");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm__tmp_multiple() {
        let outcome = evaluate_expect_outcome("rm /tmp/file1 /tmp/file2");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm__tmp_path_traversal() {
        let outcome = evaluate_expect_outcome("rm /tmp/../etc/passwd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _rm__tmp_mixed() {
        let outcome = evaluate_expect_outcome("rm /tmp/file.txt /home/user/file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _ls() {
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_rm() {
        // git rm is Allow via git_approval
        let outcome = evaluate_expect_outcome("git rm file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_rm_r() {
        let outcome = evaluate_expect_outcome("git rm -r dir/");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _echo_rm() {
        let outcome = evaluate_expect_outcome("echo rm is blocked");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _grep_r() {
        let outcome = evaluate_expect_outcome("grep -r rm .");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _cat() {
        let outcome = evaluate_expect_outcome("cat file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _mv() {
        let reason = evaluate_expect_skip("mv old.txt new.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _cargo_rm() {
        let reason = evaluate_expect_skip("cargo rm some-dep");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _xargs_rm() {
        // echo | xargs rm — echo is Allow via safe_rules, xargs is not matched
        // but rm is not the command name here (xargs is), so rm rule doesn't fire
        let reason = evaluate_expect_skip("echo file | xargs rm");
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }
}
