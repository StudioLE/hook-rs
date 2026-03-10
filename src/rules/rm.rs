//! Deny rule for `rm` to prevent file deletion.

use crate::prelude::*;

/// Deny all `rm` invocations, directing to `git rm` or `git clean` instead.
#[must_use]
pub fn rm_rule() -> SimpleRule {
    SimpleRule::new(
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
    use insta::assert_yaml_snapshot;

    #[test]
    fn rm_r() {
        let result = evaluate("rm -r /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_cap_r() {
        let result = evaluate("rm -R /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_rf() {
        let result = evaluate("rm -rf /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_cap_rf() {
        let result = evaluate("rm -Rf /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_fr() {
        let result = evaluate("rm -fr /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_f_cap_r() {
        let result = evaluate("rm -fR /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_recursive() {
        let result = evaluate("rm --recursive /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_rfi() {
        let result = evaluate("rm -rfi /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_ir() {
        let result = evaluate("rm -ir /path/to/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_single_file() {
        let result = evaluate("rm file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_multiple_files() {
        let result = evaluate("rm file1.txt file2.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_f() {
        let result = evaluate("rm -f file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_i() {
        let result = evaluate("rm -i file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_with_path() {
        let result = evaluate("rm /path/to/file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn rm_wildcard() {
        let result = evaluate("rm *.tmp");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn chained_rm_r() {
        let result = evaluate("ls && rm -r /path");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn or_chain_rm_rf() {
        let result = evaluate("false || rm -rf /tmp/nothing");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn semicolon_rm_r() {
        let result = evaluate("echo hi ; rm -r /path");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn chained_rm_file() {
        let result = evaluate("ls && rm file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn for_do_rm() {
        let result = evaluate("for f in *.tmp; do rm $f; done");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn if_then_rm() {
        let result = evaluate("if true; then rm file.txt; fi");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn if_else_rm() {
        let result = evaluate("if false; then echo hi; else rm file.txt; fi");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn while_do_rm_rf() {
        let result = evaluate("while true; do rm -rf /tmp/nothing; done");
        assert_yaml_snapshot!(result);
    }
    #[test]
    fn tmp_file_denied() {
        let result = evaluate("rm /tmp/file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tmp_f_denied() {
        let result = evaluate("rm -f /tmp/file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tmp_rf_denied() {
        let result = evaluate("rm -rf /tmp/dir");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tmp_multiple_denied() {
        let result = evaluate("rm /tmp/file1 /tmp/file2");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tmp_path_traversal_denied() {
        let result = evaluate("rm /tmp/../etc/passwd");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn tmp_mixed_non_tmp_denied() {
        let result = evaluate("rm /tmp/file.txt /home/user/file.txt");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn ls_passthrough() {
        let result = evaluate("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_rm_passthrough() {
        // git rm is Allow via git_approval
        let result = evaluate("git rm file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_rm_r_passthrough() {
        let result = evaluate("git rm -r dir/").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_rm_passthrough() {
        let result = evaluate("echo rm is blocked").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_r_passthrough() {
        let result = evaluate("grep -r rm .").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn cat_passthrough() {
        let result = evaluate("cat file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn mv_passthrough() {
        assert_eq!(evaluate("mv old.txt new.txt"), None);
    }

    #[test]
    fn cargo_rm_passthrough() {
        assert_eq!(evaluate("cargo rm some-dep"), None);
    }

    #[test]
    fn xargs_rm_passthrough() {
        // echo | xargs rm — echo is Allow via safe_rules, xargs is not matched
        // but rm is not the command name here (xargs is), so rm rule doesn't fire
        assert_eq!(evaluate("echo file | xargs rm"), None);
    }
}
