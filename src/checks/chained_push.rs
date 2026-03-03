//! Check for chained `git push` commands.

use crate::prelude::*;

/// Deny `git push` when chained with other commands.
#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    let has_git_push = parsed
        .all_commands()
        .any(|cmd| cmd.name == "git" && cmd.args.first().is_some_and(|a| a == "push"));
    if !has_git_push {
        return None;
    }
    if parsed.is_standalone() {
        None
    } else {
        Some(CheckResult::deny(
            "Chained git push is blocked. Run 'git push' as a separate, standalone command.",
        ))
    }
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
    fn add_commit_push() {
        assert_yaml_snapshot!(check("git add file.txt && git commit -m 'msg' && git push"));
    }

    #[test]
    fn commit_push_no_space() {
        assert_yaml_snapshot!(check("git commit -m 'msg'&& git push"));
    }

    #[test]
    fn commit_push_with_remote() {
        assert_yaml_snapshot!(check("git commit -m 'msg' && git push origin main"));
    }

    #[test]
    fn pull_push() {
        assert_yaml_snapshot!(check("git pull && git push"));
    }

    #[test]
    fn commit_or_push() {
        assert_yaml_snapshot!(check("git commit -m 'msg' || git push"));
    }

    #[test]
    fn commit_semicolon_push() {
        assert_yaml_snapshot!(check("git commit -m 'msg' ; git push"));
    }

    #[test]
    fn standalone_push_passthrough() {
        assert_eq!(check("git push"), None);
    }

    #[test]
    fn push_origin_main_passthrough() {
        assert_eq!(check("git push origin main"), None);
    }

    #[test]
    fn push_u_passthrough() {
        assert_eq!(check("git push -u origin feature-branch"), None);
    }

    #[test]
    fn push_set_upstream_passthrough() {
        assert_eq!(check("git push --set-upstream origin branch"), None);
    }

    #[test]
    fn push_force_with_lease_passthrough() {
        assert_eq!(check("git push --force-with-lease"), None);
    }

    #[test]
    fn git_status_passthrough() {
        assert_eq!(check("git status"), None);
    }

    #[test]
    fn commit_with_push_in_message_passthrough() {
        assert_eq!(check("git commit -m 'push changes'"), None);
    }

    #[test]
    fn echo_git_push_passthrough() {
        assert_eq!(check("echo git push"), None);
    }
}
