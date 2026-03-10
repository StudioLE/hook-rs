use crate::prelude::*;

pub fn find_rules() -> Vec<SimpleRule> {
    vec![
        SimpleRule {
            prefix: "find".to_owned(),
            with_option: Some(vec!["-delete".to_owned()]),
            outcome: Outcome::deny(
                "find -delete is blocked. Use 'find ... -print' to preview matches first, \
                 then delete with targeted commands.",
            ),
            ..Default::default()
        },
        SimpleRule {
            prefix: "find".to_owned(),
            condition: Some(has_exec_rm),
            outcome: Outcome::deny(
                "find -exec rm is blocked. Use 'find ... -print' to preview matches first, \
                 then delete with targeted commands.",
            ),
            ..Default::default()
        },
    ]
}

fn has_exec_rm(cmd: &SimpleContext) -> bool {
    cmd.args.iter().enumerate().any(|(i, arg)| {
        (arg == "-exec" || arg == "-execdir") && cmd.args.get(i + 1).is_some_and(|a| a == "rm")
    })
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn find_delete() {
        assert_yaml_snapshot!(eval("find . -name '*.tmp' -delete"));
    }

    #[test]
    fn find_path_delete() {
        assert_yaml_snapshot!(eval("find /path -type f -delete"));
    }

    #[test]
    fn find_delete_redirect() {
        assert_yaml_snapshot!(eval("find . -name .lock -delete 2>/dev/null"));
    }

    #[test]
    fn find_exec_rm() {
        assert_yaml_snapshot!(eval("find . -name '*.tmp' -exec rm {} \\;"));
    }

    #[test]
    fn find_exec_rm_f() {
        assert_yaml_snapshot!(eval("find . -type f -exec rm -f {} +"));
    }

    #[test]
    fn find_execdir_rm() {
        assert_yaml_snapshot!(eval("find . -name '*.log' -execdir rm {} \\;"));
    }

    #[test]
    fn chained_find_delete() {
        assert_yaml_snapshot!(eval("ls && find . -delete"));
    }

    #[test]
    fn semicolon_find_delete() {
        assert_yaml_snapshot!(eval("echo test ; find . -name '*.tmp' -delete"));
    }

    #[test]
    fn find_name_passthrough() {
        assert_eq!(eval("find . -name '*.rs'"), None);
    }

    #[test]
    fn find_print_passthrough() {
        assert_eq!(eval("find . -type f -print"), None);
    }

    #[test]
    fn find_maxdepth_passthrough() {
        assert_eq!(eval("find /path -maxdepth 1"), None);
    }

    #[test]
    fn find_exec_ls_passthrough() {
        assert_eq!(eval("find . -name '*.tmp' -exec ls {} \\;"), None);
    }

    #[test]
    fn find_exec_cat_passthrough() {
        assert_eq!(eval("find . -name '*.txt' -exec cat {} +"), None);
    }

    #[test]
    fn echo_find_delete_passthrough() {
        let result = eval("echo 'find -delete is dangerous'").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn grep_delete_passthrough() {
        let result = eval("grep -r 'delete' .").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_log_grep_delete_passthrough() {
        // git log is Allow via git_approval, grep is Allow via safe_rules
        let result = eval("git log --oneline | grep delete").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }
}
