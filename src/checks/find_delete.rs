//! Check for `find -delete` and `find -exec rm` commands.

use crate::prelude::*;

/// Deny `find -delete` and `find -exec rm` commands.
#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    for cmd in parsed.all_commands() {
        if cmd.name != "find" {
            continue;
        }
        if cmd.args.iter().any(|a| a == "-delete") {
            return Some(CheckResult::deny(
                "find -delete is blocked. Use 'find ... -print' to preview matches first, then delete with targeted commands.",
            ));
        }
        for (i, arg) in cmd.args.iter().enumerate() {
            if (arg == "-exec" || arg == "-execdir")
                && cmd.args.get(i + 1).is_some_and(|a| a == "rm")
            {
                return Some(CheckResult::deny(
                    "find -exec rm is blocked. Use 'find ... -print' to preview matches first, then delete with targeted commands.",
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check(command: &str) -> Option<CheckResult> {
        let parsed = parse(command)?;
        super::check(&parsed)
    }

    #[test]
    fn find_delete() {
        assert_yaml_snapshot!(check("find . -name '*.tmp' -delete"));
    }

    #[test]
    fn find_path_delete() {
        assert_yaml_snapshot!(check("find /path -type f -delete"));
    }

    #[test]
    fn find_delete_redirect() {
        assert_yaml_snapshot!(check("find . -name .lock -delete 2>/dev/null"));
    }

    #[test]
    fn find_exec_rm() {
        assert_yaml_snapshot!(check("find . -name '*.tmp' -exec rm {} \\;"));
    }

    #[test]
    fn find_exec_rm_f() {
        assert_yaml_snapshot!(check("find . -type f -exec rm -f {} +"));
    }

    #[test]
    fn find_execdir_rm() {
        assert_yaml_snapshot!(check("find . -name '*.log' -execdir rm {} \\;"));
    }

    #[test]
    fn chained_find_delete() {
        assert_yaml_snapshot!(check("ls && find . -delete"));
    }

    #[test]
    fn semicolon_find_delete() {
        assert_yaml_snapshot!(check("echo test ; find . -name '*.tmp' -delete"));
    }

    #[test]
    fn find_name_passthrough() {
        assert_eq!(check("find . -name '*.rs'"), None);
    }

    #[test]
    fn find_print_passthrough() {
        assert_eq!(check("find . -type f -print"), None);
    }

    #[test]
    fn find_maxdepth_passthrough() {
        assert_eq!(check("find /path -maxdepth 1"), None);
    }

    #[test]
    fn find_exec_ls_passthrough() {
        assert_eq!(check("find . -name '*.tmp' -exec ls {} \\;"), None);
    }

    #[test]
    fn find_exec_cat_passthrough() {
        assert_eq!(check("find . -name '*.txt' -exec cat {} +"), None);
    }

    #[test]
    fn echo_find_delete_passthrough() {
        assert_eq!(check("echo 'find -delete is dangerous'"), None);
    }

    #[test]
    fn grep_delete_passthrough() {
        assert_eq!(check("grep -r 'delete' ."), None);
    }

    #[test]
    fn git_log_grep_delete_passthrough() {
        assert_eq!(check("git log --oneline | grep delete"), None);
    }
}
