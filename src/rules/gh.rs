//! Rules for GitHub CLI commands, distinguishing read vs write operations.

use std::process::Command;

use crate::prelude::*;

/// Allow read-only `gh` operations, ask for write operations, and handle bot contexts.
pub fn gh_rules() -> Vec<SimpleRule> {
    vec![
        SimpleRule::new("gh run list", Outcome::allow("Read-only gh run list")),
        SimpleRule::new("gh run view", Outcome::allow("Read-only gh run view")),
        SimpleRule::new(
            "gh release list",
            Outcome::allow("Read-only gh release list"),
        ),
        // gh pr comment (bot in StudioLE) → Allow
        SimpleRule {
            prefix: "gh pr".to_owned(),
            condition: Some(is_bot_pr_comment),
            outcome: Outcome::allow("PR comment"),
            ..Default::default()
        },
        // gh pr comment (other) → Ask
        SimpleRule {
            prefix: "gh pr".to_owned(),
            condition: Some(is_pr_comment),
            outcome: Outcome::ask("PR comment requires approval"),
            ..Default::default()
        },
        // gh api graphql mutation → Ask
        SimpleRule {
            prefix: "gh api".to_owned(),
            condition: Some(is_graphql_mutation),
            outcome: Outcome::ask("GitHub GraphQL mutation"),
            ..Default::default()
        },
        // gh api graphql query → Allow
        SimpleRule {
            prefix: "gh api".to_owned(),
            condition: Some(is_graphql_query),
            outcome: Outcome::allow("Read-only GraphQL query"),
            ..Default::default()
        },
        // gh api with data flags (bot PR reply in StudioLE) → Allow
        SimpleRule {
            prefix: "gh api".to_owned(),
            condition: Some(is_bot_pr_reply),
            outcome: Outcome::allow("PR comment reply"),
            ..Default::default()
        },
        // gh api with data flags → Ask
        SimpleRule {
            prefix: "gh api".to_owned(),
            condition: Some(has_data_flags),
            outcome: Outcome::ask("GitHub API request with data flags"),
            ..Default::default()
        },
        // gh api -X POST/PUT/PATCH/DELETE → Ask
        SimpleRule {
            prefix: "gh api".to_owned(),
            condition: Some(has_write_method),
            outcome: Outcome::ask("GitHub API write method"),
            ..Default::default()
        },
        // gh api (catch-all read) → Allow
        SimpleRule::new("gh api", Outcome::allow("Read-only gh api command")),
    ]
}

fn is_pr_comment(cmd: &SimpleContext) -> bool {
    cmd.args.first().is_some_and(|a| a == "pr") && cmd.args.get(1).is_some_and(|a| a == "comment")
}

fn is_bot_pr_comment(cmd: &SimpleContext) -> bool {
    if !is_pr_comment(cmd) {
        return false;
    }
    let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
    let gh_user = get_gh_user().unwrap_or_default();
    let repo = extract_repo_flag(&args).unwrap_or_else(|| get_current_repo().unwrap_or_default());
    repo.starts_with("StudioLE/") && gh_user == "StudioLE-Bot"
}

fn is_graphql_query(cmd: &SimpleContext) -> bool {
    cmd.args.get(1).is_some_and(|a| a == "graphql")
}

fn is_graphql_mutation(cmd: &SimpleContext) -> bool {
    cmd.args.get(1).is_some_and(|a| a == "graphql")
        && cmd
            .args
            .iter()
            .any(|a| a.to_lowercase().contains("mutation"))
}

fn has_data_flags(cmd: &SimpleContext) -> bool {
    cmd.args.iter().any(|a| {
        matches!(
            a.as_str(),
            "-d" | "--data" | "-f" | "--field" | "-F" | "--raw-field" | "--input"
        ) || a.starts_with("--data=")
            || a.starts_with("--field=")
            || a.starts_with("--input=")
    })
}

fn is_bot_pr_reply(cmd: &SimpleContext) -> bool {
    if !has_data_flags(cmd) {
        return false;
    }
    let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
    let gh_user = get_gh_user().unwrap_or_default();
    let repo = extract_api_repo(&args).unwrap_or_default();
    has_pr_reply_path(&args) && repo.starts_with("StudioLE/") && gh_user == "StudioLE-Bot"
}

fn has_write_method(cmd: &SimpleContext) -> bool {
    cmd.args.iter().enumerate().any(|(i, arg)| {
        arg == "-X"
            && cmd.args.get(i + 1).is_some_and(|m| {
                matches!(
                    m.to_uppercase().as_str(),
                    "POST" | "PUT" | "PATCH" | "DELETE"
                )
            })
    })
}

fn extract_repo_flag(args: &[&str]) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if matches!(*arg, "-R" | "--repo") {
            return args.get(i + 1).map(|s| unquote(s));
        }
    }
    None
}

fn extract_api_repo(args: &[&str]) -> Option<String> {
    for arg in args {
        let unquoted = unquote(arg);
        let s = unquoted.strip_prefix('/').unwrap_or(&unquoted);
        if let Some(rest) = s.strip_prefix("repos/") {
            let parts: Vec<&str> = rest.splitn(3, '/').collect();
            if let (Some(owner), Some(repo), Some(_)) = (parts.first(), parts.get(1), parts.get(2))
            {
                return Some(format!("{owner}/{repo}"));
            }
        }
    }
    None
}

fn has_pr_reply_path(args: &[&str]) -> bool {
    args.iter().any(|arg| {
        let unquoted = unquote(arg);
        unquoted.contains("/pulls/")
            && unquoted.contains("/comments/")
            && unquoted.contains("/replies")
    })
}

fn get_gh_user() -> Option<String> {
    Command::new("gh")
        .args(["api", "user", "--jq", ".login"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn get_current_repo() -> Option<String> {
    Command::new("gh")
        .args([
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "--jq",
            ".nameWithOwner",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn non_gh_passthrough() {
        // ls and echo are Allow via safe_rules
        let result = evaluate("ls -la").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
        let result = evaluate("echo hello").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
        // git status is Allow via git_approval
        let result = evaluate("git status").expect("should match");
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn gh_non_api_passthrough() {
        assert_eq!(evaluate("gh pr list"), None);
        assert_eq!(evaluate("gh issue view 123"), None);
        assert_eq!(evaluate("gh repo view"), None);
    }

    #[test]
    fn gh_run_list() {
        let result = evaluate("gh run list");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_run_list_flags() {
        let result = evaluate("gh run list --limit 10");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_run_view() {
        let result = evaluate("gh run view 12345");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_run_view_log() {
        let result = evaluate("gh run view 12345 --log");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_release_list() {
        let result = evaluate("gh release list");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_release_list_flags() {
        let result = evaluate("gh release list --limit 10");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_user() {
        let result = evaluate("gh api user");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_repos() {
        let result = evaluate("gh api repos/owner/repo");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_pulls() {
        let result = evaluate("gh api repos/owner/repo/pulls");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_post() {
        let result = evaluate("gh api -X POST /repos/owner/repo/issues");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_put() {
        let result = evaluate("gh api -X PUT /repos/owner/repo/issues/1");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_patch() {
        let result = evaluate("gh api -X PATCH /repos/owner/repo/issues/1");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_delete() {
        let result = evaluate("gh api -X DELETE /repos/owner/repo/issues/1");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_pipe_base64() {
        let result = evaluate("gh api repos/USER/REPO/readme --jq .content 2>&1 | base64 -d");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_pipe_jq() {
        let result = evaluate("gh api repos/owner/repo/pulls | jq -r '.[].title'");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_jq_pipe_in_jq() {
        let result = evaluate("gh api repos/owner/repo/readme --jq '.content | @base64d'");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_d_before_jq() {
        let result = evaluate("gh api repos/owner/repo -d @body.json --jq '.content | @base64d'");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_graphql_read() {
        let result = evaluate(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussions(first: 10) { nodes { title } } } }'",
        );
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_graphql_with_jq() {
        let result = evaluate(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussion(number: 97) { author { login } } } }' --jq '.data.repository.discussion'",
        );
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_graphql_explicit_query() {
        let result = evaluate("gh api graphql -f query='query { viewer { login } }'");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_graphql_mutation() {
        let result = evaluate(
            "gh api graphql -f query='mutation { addComment(input: {subjectId: \"123\", body: \"test\"}) { commentEdge { node { body } } } }'",
        );
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_f_flag() {
        let result = evaluate("gh api /repos/owner/repo/issues -f title=test");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_cap_f_flag() {
        let result = evaluate("gh api /repos/owner/repo/issues -F body=test");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_field_flag() {
        let result = evaluate("gh api /repos/owner/repo/issues --field title=test");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_d_flag() {
        let result = evaluate("gh api /repos/owner/repo -d @body.json");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_data_flag() {
        let result = evaluate("gh api /repos/owner/repo --data @body.json");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_api_input_flag() {
        let result = evaluate("gh api /repos/owner/repo --input file.json");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_pr_comment() {
        let result = evaluate("gh pr comment 123 --body 'test'");
        assert_yaml_snapshot!(result);
    }

    #[test]
    fn gh_pr_comment_bot_studiole() {
        let parsed =
            CompleteContext::parse("gh pr comment 123 --body 'test'").expect("should parse");
        let cmd = parsed.all_commands().next().expect("should have command");
        let is_bot = super::is_bot_pr_comment(cmd);
        let is_comment = super::is_pr_comment(cmd);
        assert!(is_comment);
        // is_bot depends on live gh auth — just verify the function runs
        let _ = is_bot;
    }

    #[test]
    fn gh_pr_comment_bot_with_repo_flag() {
        let parsed =
            CompleteContext::parse("gh pr comment 123 --body 'test' -R StudioLE/some-repo")
                .expect("should parse");
        let cmd = parsed.all_commands().next().expect("should have command");
        assert!(super::is_pr_comment(cmd));
        let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
        let repo = super::extract_repo_flag(&args);
        assert_eq!(repo, Some("StudioLE/some-repo".to_owned()));
    }

    #[test]
    fn gh_api_pr_reply_bot_studiole() {
        let parsed = CompleteContext::parse(
            "gh api repos/StudioLE/some-repo/pulls/1/comments/2/replies -d '{\"body\":\"test\"}'",
        )
        .expect("should parse");
        let cmd = parsed.all_commands().next().expect("should have command");
        assert!(super::has_data_flags(cmd));
        let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
        assert!(super::has_pr_reply_path(&args));
        let repo = super::extract_api_repo(&args);
        assert_eq!(repo, Some("StudioLE/some-repo".to_owned()));
    }
}
