use regex::Regex;
use std::sync::LazyLock;

use crate::command;
use crate::prelude::*;

static RM_CMD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|&&|\|\||[;|]|\bdo\b|\bthen\b|\belse\b)\s*rm\s").expect("valid regex")
});

static TMP_RM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*rm\s+(-[a-zA-Z]+\s+)*(/tmp/\S+\s*)+$").expect("valid regex"));

static RECURSIVE_RM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|&&|\|\||[;|]|\bdo\b|\bthen\b|\belse\b)\s*rm\s+(-[a-zA-Z]*[rR]|--recursive)")
        .expect("valid regex")
});

pub fn check(command: &str) -> Option<CheckResult> {
    // Allow standalone rm of /tmp/ paths (no path traversal)
    if TMP_RM.is_match(command) && !command.contains("..") {
        return None;
    }
    if RECURSIVE_RM.is_match(command) {
        return Some(CheckResult::deny(
            "Recursive rm is blocked. Use 'git rm -r <dir>' for tracked directories, \
             'git clean -fd <dir>' for untracked directories (or -fxd if gitignored), \
             or 'rmdir' for empty directories.",
        ));
    }
    if RM_CMD.is_match(command) {
        return Some(CheckResult::deny(
            "rm is blocked. Use 'git rm <file>' for tracked files or \
             'git clean -f <file>' for untracked files (or -fx if gitignored).",
        ));
    }
    if has_git_clean_d(command) {
        return Some(CheckResult::deny(
            "git clean with -d is blocked. Use 'git clean -f <file>' for specific files \
             (or -fx if gitignored) or 'git rm -r <dir>' for tracked directories.",
        ));
    }
    None
}

fn has_git_clean_d(command: &str) -> bool {
    for args in command::git_args_in_segments(command) {
        let mut parts = args.split_whitespace();
        if parts.next() != Some("clean") {
            continue;
        }
        for part in parts {
            if part.starts_with('-') && !part.starts_with("--") && part.contains('d') {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn rm_r() {
        assert_yaml_snapshot!(check("rm -r /path/to/dir"));
    }

    #[test]
    fn rm_cap_r() {
        assert_yaml_snapshot!(check("rm -R /path/to/dir"));
    }

    #[test]
    fn rm_rf() {
        assert_yaml_snapshot!(check("rm -rf /path/to/dir"));
    }

    #[test]
    fn rm_cap_rf() {
        assert_yaml_snapshot!(check("rm -Rf /path/to/dir"));
    }

    #[test]
    fn rm_fr() {
        assert_yaml_snapshot!(check("rm -fr /path/to/dir"));
    }

    #[test]
    fn rm_f_cap_r() {
        assert_yaml_snapshot!(check("rm -fR /path/to/dir"));
    }

    #[test]
    fn rm_recursive() {
        assert_yaml_snapshot!(check("rm --recursive /path/to/dir"));
    }

    #[test]
    fn rm_rfi() {
        assert_yaml_snapshot!(check("rm -rfi /path/to/dir"));
    }

    #[test]
    fn rm_ir() {
        assert_yaml_snapshot!(check("rm -ir /path/to/dir"));
    }

    #[test]
    fn rm_single_file() {
        assert_yaml_snapshot!(check("rm file.txt"));
    }

    #[test]
    fn rm_multiple_files() {
        assert_yaml_snapshot!(check("rm file1.txt file2.txt"));
    }

    #[test]
    fn rm_f() {
        assert_yaml_snapshot!(check("rm -f file.txt"));
    }

    #[test]
    fn rm_i() {
        assert_yaml_snapshot!(check("rm -i file.txt"));
    }

    #[test]
    fn rm_with_path() {
        assert_yaml_snapshot!(check("rm /path/to/file.txt"));
    }

    #[test]
    fn rm_wildcard() {
        assert_yaml_snapshot!(check("rm *.tmp"));
    }

    #[test]
    fn chained_rm_r() {
        assert_yaml_snapshot!(check("ls && rm -r /path"));
    }

    #[test]
    fn or_chain_rm_rf() {
        assert_yaml_snapshot!(check("false || rm -rf /path"));
    }

    #[test]
    fn semicolon_rm_r() {
        assert_yaml_snapshot!(check("echo hi ; rm -r /path"));
    }

    #[test]
    fn chained_rm_file() {
        assert_yaml_snapshot!(check("ls && rm file.txt"));
    }

    #[test]
    fn for_do_rm() {
        assert_yaml_snapshot!(check("for f in *.tmp; do rm $f; done"));
    }

    #[test]
    fn if_then_rm() {
        assert_yaml_snapshot!(check("if true; then rm file.txt; fi"));
    }

    #[test]
    fn if_else_rm() {
        assert_yaml_snapshot!(check("if false; then echo hi; else rm file.txt; fi"));
    }

    #[test]
    fn while_do_rm_rf() {
        assert_yaml_snapshot!(check("while true; do rm -rf /path; done"));
    }

    #[test]
    fn tmp_file_passthrough() {
        assert_eq!(check("rm /tmp/file.txt"), None);
    }

    #[test]
    fn tmp_f_passthrough() {
        assert_eq!(check("rm -f /tmp/file.txt"), None);
    }

    #[test]
    fn tmp_rf_passthrough() {
        assert_eq!(check("rm -rf /tmp/dir"), None);
    }

    #[test]
    fn tmp_multiple_passthrough() {
        assert_eq!(check("rm /tmp/file1 /tmp/file2"), None);
    }

    #[test]
    fn tmp_path_traversal() {
        assert_yaml_snapshot!(check("rm /tmp/../etc/passwd"));
    }

    #[test]
    fn tmp_mixed_non_tmp() {
        assert_yaml_snapshot!(check("rm /tmp/file.txt /home/user/file.txt"));
    }

    #[test]
    fn ls_passthrough() {
        assert_eq!(check("ls -la"), None);
    }

    #[test]
    fn git_rm_passthrough() {
        assert_eq!(check("git rm file.txt"), None);
    }

    #[test]
    fn git_rm_r_passthrough() {
        assert_eq!(check("git rm -r dir/"), None);
    }

    #[test]
    fn echo_rm_passthrough() {
        assert_eq!(check("echo rm is blocked"), None);
    }

    #[test]
    fn grep_r_passthrough() {
        assert_eq!(check("grep -r rm ."), None);
    }

    #[test]
    fn cat_passthrough() {
        assert_eq!(check("cat file.txt"), None);
    }

    #[test]
    fn mv_passthrough() {
        assert_eq!(check("mv old.txt new.txt"), None);
    }

    #[test]
    fn cargo_rm_passthrough() {
        assert_eq!(check("cargo rm some-dep"), None);
    }

    #[test]
    fn xargs_rm_passthrough() {
        assert_eq!(check("echo file | xargs rm"), None);
    }

    #[test]
    fn git_clean_fd() {
        assert_yaml_snapshot!(check("git clean -fd"));
    }

    #[test]
    fn git_clean_fxd() {
        assert_yaml_snapshot!(check("git clean -fxd"));
    }

    #[test]
    fn git_clean_d() {
        assert_yaml_snapshot!(check("git clean -d"));
    }

    #[test]
    fn git_clean_df() {
        assert_yaml_snapshot!(check("git clean -df"));
    }

    #[test]
    fn git_clean_dxf() {
        assert_yaml_snapshot!(check("git clean -dxf"));
    }

    #[test]
    fn chained_git_clean_fd() {
        assert_yaml_snapshot!(check("ls && git clean -fd"));
    }

    #[test]
    fn git_clean_f_passthrough() {
        assert_eq!(check("git clean -f file.txt"), None);
    }

    #[test]
    fn git_clean_fx_passthrough() {
        assert_eq!(check("git clean -fx file.txt"), None);
    }

    #[test]
    fn git_clean_fx_dash_in_filename_passthrough() {
        assert_eq!(
            check("git clean -fx /path/to/some-dash-delimited-file.sh"),
            None
        );
    }

    #[test]
    fn git_clean_f_dash_in_path_passthrough() {
        assert_eq!(check("git clean -f /path/dir-name/file.txt"), None);
    }

    #[test]
    fn git_clean_n_passthrough() {
        assert_eq!(check("git clean -n"), None);
    }

    #[test]
    fn echo_git_clean_passthrough() {
        assert_eq!(check("echo git clean -fxd"), None);
    }

    #[test]
    fn c_path_git_clean_fd() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura clean -fd"));
    }

    #[test]
    fn c_path_git_clean_fxd() {
        assert_yaml_snapshot!(check("git -C /var/mnt/e/Repos/Rust/caesura clean -fxd"));
    }

    #[test]
    fn c_path_quoted_git_clean_fd() {
        assert_yaml_snapshot!(check("git -C \"/var/mnt/e/Repos/Rust/caesura\" clean -fd"));
    }

    #[test]
    fn c_path_git_clean_f_passthrough() {
        assert_eq!(
            check("git -C /var/mnt/e/Repos/Rust/caesura clean -f file.txt"),
            None
        );
    }
}
