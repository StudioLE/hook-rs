//! Evaluate GitHub CLI commands for safety.

use std::process::Command;

use crate::prelude::*;

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

/// Evaluate a `gh` command for safety using live GitHub context.
#[must_use]
pub fn check(parsed: &ParsedCommand) -> Option<CheckResult> {
    let gh_user = get_gh_user().unwrap_or_default();
    let current_repo = get_current_repo().unwrap_or_default();
    check_with_context(parsed, &gh_user, &current_repo)
}

/// Evaluate a `gh` command with pre-fetched user and repo context.
#[must_use]
pub fn check_with_context(
    parsed: &ParsedCommand,
    gh_user: &str,
    current_repo: &str,
) -> Option<CheckResult> {
    // Find the first command in any pipeline that is `gh`
    // (only first in pipeline — piped commands like `| base64` are ignored)
    let cmd = parsed
        .and_or_lists
        .iter()
        .flat_map(|aol| &aol.items)
        .filter_map(|pi| pi.commands.first())
        .find(|c| c.name == "gh")?;

    let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();

    // gh pr comment
    if args.first() == Some(&"pr") && args.get(1) == Some(&"comment") {
        let repo = extract_repo_flag(&args).unwrap_or_else(|| current_repo.to_owned());
        if repo.starts_with("StudioLE/") && gh_user == "StudioLE-Bot" {
            return Some(CheckResult::allow("PR comment"));
        }
        return Some(CheckResult::ask(format!(
            "PR comment as {gh_user} in {repo}"
        )));
    }

    // gh run list/view
    if args.first() == Some(&"run") && matches!(args.get(1).copied(), Some("list" | "view")) {
        return Some(CheckResult::allow("Read-only gh run command"));
    }

    // gh release list
    if args.first() == Some(&"release") && args.get(1) == Some(&"list") {
        return Some(CheckResult::allow("Read-only gh release list command"));
    }

    // Must be gh api
    if args.first() != Some(&"api") {
        return None;
    }

    // gh api graphql
    if args.get(1) == Some(&"graphql") {
        if parsed.raw.to_lowercase().contains("mutation") {
            return Some(CheckResult::ask("GitHub GraphQL mutation"));
        }
        return Some(CheckResult::allow("Read-only GraphQL query"));
    }

    // Data flags: -d, --data, -f, --field, -F, --raw-field, --input
    let has_data_flags = args.iter().any(|a| {
        matches!(
            *a,
            "-d" | "--data" | "-f" | "--field" | "-F" | "--raw-field" | "--input"
        ) || a.starts_with("--data=")
            || a.starts_with("--field=")
            || a.starts_with("--input=")
    });
    if has_data_flags {
        let repo = extract_api_repo(&args).unwrap_or_default();
        if has_pr_reply_path(&args) && repo.starts_with("StudioLE/") && gh_user == "StudioLE-Bot" {
            return Some(CheckResult::allow("PR comment reply"));
        }
        return Some(CheckResult::ask(format!(
            "GitHub API requests with data flags as {gh_user} in {repo}"
        )));
    }

    // Write method: -X POST/PUT/PATCH/DELETE
    for (i, arg) in args.iter().enumerate() {
        if *arg == "-X"
            && let Some(method) = args.get(i + 1)
        {
            let upper = method.to_uppercase();
            if matches!(upper.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
                return Some(CheckResult::ask(format!("GitHub API {upper}")));
            }
            break;
        }
    }

    Some(CheckResult::allow("Read-only gh api command"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check_t(command: &str) -> Option<CheckResult> {
        let parsed = parse(command)?;
        check_with_context(&parsed, "test-user", "owner/repo")
    }

    fn check_bot(command: &str) -> Option<CheckResult> {
        let parsed = parse(command)?;
        check_with_context(&parsed, "StudioLE-Bot", "StudioLE/some-repo")
    }

    #[test]
    fn non_gh_passthrough() {
        assert_eq!(check_t("ls -la"), None);
        assert_eq!(check_t("git status"), None);
        assert_eq!(check_t("echo hello"), None);
    }

    #[test]
    fn gh_non_api_passthrough() {
        assert_eq!(check_t("gh pr list"), None);
        assert_eq!(check_t("gh issue view 123"), None);
        assert_eq!(check_t("gh repo view"), None);
    }

    #[test]
    fn gh_run_list() {
        assert_yaml_snapshot!(check_t("gh run list"));
    }

    #[test]
    fn gh_run_list_flags() {
        assert_yaml_snapshot!(check_t("gh run list --limit 10"));
    }

    #[test]
    fn gh_run_view() {
        assert_yaml_snapshot!(check_t("gh run view 12345"));
    }

    #[test]
    fn gh_run_view_log() {
        assert_yaml_snapshot!(check_t("gh run view 12345 --log"));
    }

    #[test]
    fn gh_release_list() {
        assert_yaml_snapshot!(check_t("gh release list"));
    }

    #[test]
    fn gh_release_list_flags() {
        assert_yaml_snapshot!(check_t("gh release list --limit 10"));
    }

    #[test]
    fn gh_api_user() {
        assert_yaml_snapshot!(check_t("gh api user"));
    }

    #[test]
    fn gh_api_repos() {
        assert_yaml_snapshot!(check_t("gh api repos/owner/repo"));
    }

    #[test]
    fn gh_api_pulls() {
        assert_yaml_snapshot!(check_t("gh api repos/owner/repo/pulls"));
    }

    #[test]
    fn gh_api_post() {
        assert_yaml_snapshot!(check_t("gh api -X POST /repos/owner/repo/issues"));
    }

    #[test]
    fn gh_api_put() {
        assert_yaml_snapshot!(check_t("gh api -X PUT /repos/owner/repo/issues/1"));
    }

    #[test]
    fn gh_api_patch() {
        assert_yaml_snapshot!(check_t("gh api -X PATCH /repos/owner/repo/issues/1"));
    }

    #[test]
    fn gh_api_delete() {
        assert_yaml_snapshot!(check_t("gh api -X DELETE /repos/owner/repo/issues/1"));
    }

    #[test]
    fn gh_api_pipe_base64() {
        assert_yaml_snapshot!(check_t(
            "gh api repos/USER/REPO/readme --jq .content 2>&1 | base64 -d"
        ));
    }

    #[test]
    fn gh_api_pipe_jq() {
        assert_yaml_snapshot!(check_t("gh api repos/owner/repo/pulls | jq -r '.[].title'"));
    }

    #[test]
    fn gh_api_jq_pipe_in_jq() {
        assert_yaml_snapshot!(check_t(
            "gh api repos/owner/repo/readme --jq '.content | @base64d'"
        ));
    }

    #[test]
    fn gh_api_d_before_jq() {
        assert_yaml_snapshot!(check_t(
            "gh api repos/owner/repo -d @body.json --jq '.content | @base64d'"
        ));
    }

    #[test]
    fn gh_graphql_read() {
        assert_yaml_snapshot!(check_t(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussions(first: 10) { nodes { title } } } }'"
        ));
    }

    #[test]
    fn gh_graphql_with_jq() {
        assert_yaml_snapshot!(check_t(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussion(number: 97) { author { login } } } }' --jq '.data.repository.discussion'"
        ));
    }

    #[test]
    fn gh_graphql_explicit_query() {
        assert_yaml_snapshot!(check_t(
            "gh api graphql -f query='query { viewer { login } }'"
        ));
    }

    #[test]
    fn gh_graphql_mutation() {
        assert_yaml_snapshot!(check_t(
            "gh api graphql -f query='mutation { addComment(input: {subjectId: \"123\", body: \"test\"}) { commentEdge { node { body } } } }'"
        ));
    }

    #[test]
    fn gh_api_f_flag() {
        assert_yaml_snapshot!(check_t("gh api /repos/owner/repo/issues -f title=test"));
    }

    #[test]
    fn gh_api_cap_f_flag() {
        assert_yaml_snapshot!(check_t("gh api /repos/owner/repo/issues -F body=test"));
    }

    #[test]
    fn gh_api_field_flag() {
        assert_yaml_snapshot!(check_t(
            "gh api /repos/owner/repo/issues --field title=test"
        ));
    }

    #[test]
    fn gh_api_d_flag() {
        assert_yaml_snapshot!(check_t("gh api /repos/owner/repo -d @body.json"));
    }

    #[test]
    fn gh_api_data_flag() {
        assert_yaml_snapshot!(check_t("gh api /repos/owner/repo --data @body.json"));
    }

    #[test]
    fn gh_api_input_flag() {
        assert_yaml_snapshot!(check_t("gh api /repos/owner/repo --input file.json"));
    }

    #[test]
    fn gh_pr_comment() {
        assert_yaml_snapshot!(check_t("gh pr comment 123 --body 'test'"));
    }

    #[test]
    fn gh_pr_comment_bot_studiole() {
        assert_yaml_snapshot!(check_bot("gh pr comment 123 --body 'test'"));
    }

    #[test]
    fn gh_pr_comment_bot_with_repo_flag() {
        assert_yaml_snapshot!(check_with_context(
            &parse("gh pr comment 123 --body 'test' -R StudioLE/some-repo").expect("should parse"),
            "StudioLE-Bot",
            "other/repo"
        ));
    }

    #[test]
    fn gh_api_pr_reply_bot_studiole() {
        assert_yaml_snapshot!(check_with_context(
            &parse(
                "gh api repos/StudioLE/some-repo/pulls/1/comments/2/replies -d '{\"body\":\"test\"}'"
            )
            .expect("should parse"),
            "StudioLE-Bot",
            ""
        ));
    }
}
