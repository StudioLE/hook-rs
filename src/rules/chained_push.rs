use crate::prelude::*;

pub fn chained_push_rules() -> Vec<CompleteRule> {
    vec![CompleteRule {
        condition: Some(is_chained_git_push),
        outcome: Outcome::deny(
            "Chained git push is blocked. Run 'git push' as a separate, standalone command.",
        ),
    }]
}

fn is_chained_git_push(parsed: &CompleteContext) -> bool {
    let has_git_push = parsed
        .all_commands()
        .any(|cmd| cmd.name == "git" && cmd.args.first().is_some_and(|a| a == "push"));
    has_git_push && !parsed.is_standalone()
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    fn eval(command: &str) -> Option<Outcome> {
        crate::evaluate::evaluate(command)
    }

    #[test]
    fn add_commit_push() {
        assert_yaml_snapshot!(eval("git add file.txt && git commit -m 'msg' && git push"));
    }

    #[test]
    fn commit_push_no_space() {
        assert_yaml_snapshot!(eval("git commit -m 'msg'&& git push"));
    }

    #[test]
    fn commit_push_with_remote() {
        assert_yaml_snapshot!(eval("git commit -m 'msg' && git push origin main"));
    }

    #[test]
    fn pull_push() {
        assert_yaml_snapshot!(eval("git pull && git push"));
    }

    #[test]
    fn commit_or_push() {
        assert_yaml_snapshot!(eval("git commit -m 'msg' || git push"));
    }

    #[test]
    fn commit_semicolon_push() {
        assert_yaml_snapshot!(eval("git commit -m 'msg' ; git push"));
    }

    #[test]
    fn standalone_push_passthrough() {
        assert_eq!(eval("git push"), None);
    }

    #[test]
    fn push_origin_main_passthrough() {
        assert_eq!(eval("git push origin main"), None);
    }

    #[test]
    fn push_u_passthrough() {
        assert_eq!(eval("git push -u origin feature-branch"), None);
    }

    #[test]
    fn push_set_upstream_passthrough() {
        assert_eq!(eval("git push --set-upstream origin branch"), None);
    }

    #[test]
    fn push_force_with_lease_passthrough() {
        assert_eq!(eval("git push --force-with-lease"), None);
    }

    #[test]
    fn git_status_passthrough() {
        // git status alone is Allow via git_approval, not passthrough
        let result = eval("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn commit_with_push_in_message_passthrough() {
        assert_eq!(eval("git commit -m 'push changes'"), None);
    }

    #[test]
    fn echo_git_push_passthrough() {
        // echo is Allow via safe_rules
        let result = eval("echo git push").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }
}
