use crate::prelude::*;
use crate::types::Connector;

#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    for aol in &parsed.and_or_lists {
        let (Some(first), Some(second)) = (aol.items.first(), aol.items.get(1)) else {
            continue;
        };
        if first.connector.is_none()
            && second.connector == Some(Connector::And)
            && first.commands.first().is_some_and(|c| c.name == "cd")
            && second.commands.first().is_some_and(|c| c.name == "git")
        {
            return Some(CheckResult::deny(
                "Do not chain cd and git. Use 'git -C <path> <command>' instead.",
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check(command: &str) -> Option<CheckResult> {
        let parsed = crate::command::parse(command)?;
        super::check(&parsed)
    }

    #[test]
    fn cd_and_git_status() {
        assert_yaml_snapshot!(check("cd /var/mnt/e/Repos/Rust/caesura && git status"));
    }

    #[test]
    fn cd_and_git_commit() {
        assert_yaml_snapshot!(check(
            "cd /var/mnt/e/Repos/Rust/caesura && git commit -m 'msg'"
        ));
    }

    #[test]
    fn cd_untrusted_and_git() {
        assert_yaml_snapshot!(check("cd /tmp/sketchy && git log"));
    }

    #[test]
    fn cd_relative_and_git() {
        assert_yaml_snapshot!(check("cd ../relative/path && git diff"));
    }

    #[test]
    fn cd_forked_and_git() {
        assert_yaml_snapshot!(check("cd /var/mnt/e/Repos/Forked/repo && git status"));
    }

    #[test]
    fn cd_alone_passthrough() {
        assert_eq!(check("cd /var/mnt/e/Repos/Rust/caesura"), None);
    }

    #[test]
    fn git_alone_passthrough() {
        assert_eq!(check("git status"), None);
    }

    #[test]
    fn git_log_passthrough() {
        assert_eq!(check("git log --oneline -5"), None);
    }

    #[test]
    fn non_cd_compound_passthrough() {
        assert_eq!(check("ls -la && git status"), None);
    }

    #[test]
    fn echo_cd_passthrough() {
        assert_eq!(check("echo cd /path && git status"), None);
    }
}
