use crate::prelude::*;

pub fn cd_git_rules() -> Vec<CompleteRule> {
    vec![CompleteRule {
        condition: Some(is_cd_then_git),
        outcome: Outcome::deny("Do not chain cd and git. Use 'git -C <path> <command>' instead."),
    }]
}

fn is_cd_then_git(parsed: &CompleteContext) -> bool {
    let (Some(first), Some(second)) = (parsed.children.first(), parsed.children.get(1)) else {
        return false;
    };
    first.connector.is_none()
        && second.connector == Some(Connector::And)
        && first.children.first().is_some_and(|c| c.name == "cd")
        && second.children.first().is_some_and(|c| c.name == "git")
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn cd_and_git_status() {
        assert_yaml_snapshot!(eval("cd /var/mnt/e/Repos/Rust/caesura && git status"));
    }

    #[test]
    fn cd_and_git_commit() {
        assert_yaml_snapshot!(eval(
            "cd /var/mnt/e/Repos/Rust/caesura && git commit -m 'msg'"
        ));
    }

    #[test]
    fn cd_untrusted_and_git() {
        assert_yaml_snapshot!(eval("cd /tmp/sketchy && git log"));
    }

    #[test]
    fn cd_relative_and_git() {
        assert_yaml_snapshot!(eval("cd ../relative/path && git diff"));
    }

    #[test]
    fn cd_forked_and_git() {
        assert_yaml_snapshot!(eval("cd /var/mnt/e/Repos/Forked/repo && git status"));
    }

    #[test]
    fn cd_alone_passthrough() {
        assert_eq!(eval("cd /var/mnt/e/Repos/Rust/caesura"), None);
    }

    #[test]
    fn git_alone_passthrough() {
        // git status alone is matched by git_approval as Allow, not passthrough
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn git_log_passthrough() {
        // git log is matched by git_approval as Allow
        let result = eval("git log --oneline -5").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn non_cd_compound_passthrough() {
        // ls -la is Allow via safe_rules, git status is Allow via git_approval
        let result = eval("ls -la && git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn echo_cd_allowed() {
        // echo is Allow via safe_rules, git status is Allow via git_approval
        let result = eval("echo cd /path && git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }
}
