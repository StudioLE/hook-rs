use crate::checks;
use crate::prelude::*;

type CheckFn = fn(&str) -> Option<CheckResult>;

const CHECKS: &[CheckFn] = &[
    checks::gh_cli::check,
    checks::rm::check,
    checks::git_approval::check,
    checks::cd_git::check,
    checks::git_stash::check,
    checks::git_reset::check,
    checks::git_checkout::check,
    checks::find_delete::check,
    checks::chained_push::check,
    checks::echo_separator::check,
    checks::insta_review::check,
    checks::long_python::check,
];

pub fn evaluate(command: &str) -> Option<CheckResult> {
    CHECKS.iter().find_map(|check| check(command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Decision;

    fn eval(command: &str) -> Option<CheckResult> {
        evaluate(command)
    }

    #[test]
    fn safe_git_allowed() {
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn rm_denied() {
        let result = eval("rm -rf /path").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn stash_pop_denied() {
        let result = eval("git stash pop").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn reset_hard_denied() {
        let result = eval("git reset --hard").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn checkout_discard_denied() {
        let result = eval("git checkout -- file.txt").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn chained_push_denied() {
        let result = eval("git commit -m 'msg' && git push").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn echo_separator_denied() {
        let result = eval("cmd && echo \"---\"").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn find_delete_denied() {
        let result = eval("find . -name '*.tmp' -delete").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn insta_heredoc_denied() {
        let result = eval("cargo insta review <<EOF\na\nEOF").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn cd_git_denied() {
        let result = eval("cd /path && git status").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn plain_ls_passthrough() {
        assert_eq!(eval("ls -la"), None);
    }

    #[test]
    fn plain_cargo_passthrough() {
        assert_eq!(eval("cargo build"), None);
    }

    #[test]
    fn standalone_push_passthrough() {
        assert_eq!(eval("git push"), None);
    }

    #[test]
    fn git_branch_read_allowed() {
        let result = eval("git branch -a").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_branch_write_passthrough() {
        assert_eq!(eval("git branch -d old"), None);
    }

    #[test]
    fn git_tag_read_allowed() {
        let result = eval("git tag -l").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_tag_create_passthrough() {
        assert_eq!(eval("git tag v1.0"), None);
    }

    #[test]
    fn git_remote_verbose_allowed() {
        let result = eval("git remote -v").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_remote_add_passthrough() {
        assert_eq!(eval("git remote add upstream https://x.com"), None);
    }

    #[test]
    fn tmp_rm_passthrough() {
        assert_eq!(eval("rm /tmp/file.txt"), None);
    }

    #[test]
    fn git_clean_d_denied() {
        let result = eval("git clean -fd").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
    }

    #[test]
    fn forked_path_ask() {
        let result = eval("git -C /var/mnt/e/Repos/Forked/repo status").expect("should match");
        assert_eq!(result.decision, Decision::Ask);
    }

    #[test]
    fn unknown_path_ask() {
        let result = eval("git -C /tmp/sketchy status").expect("should match");
        assert_eq!(result.decision, Decision::Ask);
    }

    #[test]
    fn rm_takes_priority_over_git_approval() {
        let result = eval("rm -rf /path").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("Recursive rm"));
    }

    #[test]
    fn c_path_stash_pop_denied() {
        let result =
            eval("git -C /var/mnt/e/Repos/Rogue/docker/caddy stash pop").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("stash pop"));
    }

    #[test]
    fn c_path_reset_hard_denied() {
        let result =
            eval("git -C /var/mnt/e/Repos/Rust/caesura reset --hard").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("reset --hard"));
    }

    #[test]
    fn c_path_checkout_discard_denied() {
        let result = eval("git -C /var/mnt/e/Repos/Rust/caesura checkout -- file.txt")
            .expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("checkout --"));
    }

    #[test]
    fn c_path_git_clean_d_denied() {
        let result = eval("git -C /var/mnt/e/Repos/Rust/caesura clean -fd").expect("should match");
        assert_eq!(result.decision, Decision::Deny);
        assert!(result.reason.contains("clean"));
    }
}
