use crate::prelude::*;

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

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn rm_r() {
        assert_yaml_snapshot!(eval("rm -r /path/to/dir"));
    }

    #[test]
    fn rm_cap_r() {
        assert_yaml_snapshot!(eval("rm -R /path/to/dir"));
    }

    #[test]
    fn rm_rf() {
        assert_yaml_snapshot!(eval("rm -rf /path/to/dir"));
    }

    #[test]
    fn rm_cap_rf() {
        assert_yaml_snapshot!(eval("rm -Rf /path/to/dir"));
    }

    #[test]
    fn rm_fr() {
        assert_yaml_snapshot!(eval("rm -fr /path/to/dir"));
    }

    #[test]
    fn rm_f_cap_r() {
        assert_yaml_snapshot!(eval("rm -fR /path/to/dir"));
    }

    #[test]
    fn rm_recursive() {
        assert_yaml_snapshot!(eval("rm --recursive /path/to/dir"));
    }

    #[test]
    fn rm_rfi() {
        assert_yaml_snapshot!(eval("rm -rfi /path/to/dir"));
    }

    #[test]
    fn rm_ir() {
        assert_yaml_snapshot!(eval("rm -ir /path/to/dir"));
    }

    #[test]
    fn rm_single_file() {
        assert_yaml_snapshot!(eval("rm file.txt"));
    }

    #[test]
    fn rm_multiple_files() {
        assert_yaml_snapshot!(eval("rm file1.txt file2.txt"));
    }

    #[test]
    fn rm_f() {
        assert_yaml_snapshot!(eval("rm -f file.txt"));
    }

    #[test]
    fn rm_i() {
        assert_yaml_snapshot!(eval("rm -i file.txt"));
    }

    #[test]
    fn rm_with_path() {
        assert_yaml_snapshot!(eval("rm /path/to/file.txt"));
    }

    #[test]
    fn rm_wildcard() {
        assert_yaml_snapshot!(eval("rm *.tmp"));
    }

    #[test]
    fn chained_rm_r() {
        assert_yaml_snapshot!(eval("ls && rm -r /path"));
    }

    #[test]
    fn or_chain_rm_rf() {
        assert_yaml_snapshot!(eval("false || rm -rf /tmp/nothing"));
    }

    #[test]
    fn semicolon_rm_r() {
        assert_yaml_snapshot!(eval("echo hi ; rm -r /path"));
    }

    #[test]
    fn chained_rm_file() {
        assert_yaml_snapshot!(eval("ls && rm file.txt"));
    }

    #[test]
    fn for_do_rm() {
        assert_yaml_snapshot!(eval("for f in *.tmp; do rm $f; done"));
    }

    #[test]
    fn if_then_rm() {
        assert_yaml_snapshot!(eval("if true; then rm file.txt; fi"));
    }

    #[test]
    fn if_else_rm() {
        assert_yaml_snapshot!(eval("if false; then echo hi; else rm file.txt; fi"));
    }

    #[test]
    fn while_do_rm_rf() {
        assert_yaml_snapshot!(eval("while true; do rm -rf /tmp/nothing; done"));
    }
    #[test]
    fn tmp_file_denied() {
        assert_yaml_snapshot!(eval("rm /tmp/file.txt"));
    }

    #[test]
    fn tmp_f_denied() {
        assert_yaml_snapshot!(eval("rm -f /tmp/file.txt"));
    }

    #[test]
    fn tmp_rf_denied() {
        assert_yaml_snapshot!(eval("rm -rf /tmp/dir"));
    }

    #[test]
    fn tmp_multiple_denied() {
        assert_yaml_snapshot!(eval("rm /tmp/file1 /tmp/file2"));
    }

    #[test]
    fn tmp_path_traversal_denied() {
        assert_yaml_snapshot!(eval("rm /tmp/../etc/passwd"));
    }

    #[test]
    fn tmp_mixed_non_tmp_denied() {
        assert_yaml_snapshot!(eval("rm /tmp/file.txt /home/user/file.txt"));
    }

    #[test]
    fn ls_passthrough() {
        let result = eval("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_rm_passthrough() {
        // git rm is Allow via git_approval
        let result = eval("git rm file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_rm_r_passthrough() {
        let result = eval("git rm -r dir/").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_rm_passthrough() {
        let result = eval("echo rm is blocked").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_r_passthrough() {
        let result = eval("grep -r rm .").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn cat_passthrough() {
        let result = eval("cat file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn mv_passthrough() {
        assert_eq!(eval("mv old.txt new.txt"), None);
    }

    #[test]
    fn cargo_rm_passthrough() {
        assert_eq!(eval("cargo rm some-dep"), None);
    }

    #[test]
    fn xargs_rm_passthrough() {
        // echo | xargs rm — echo is Allow via safe_rules, xargs is not matched
        // but rm is not the command name here (xargs is), so rm rule doesn't fire
        assert_eq!(eval("echo file | xargs rm"), None);
    }
}
