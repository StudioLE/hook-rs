//! Rules for GitHub CLI commands, distinguishing read vs write operations.

use std::process::Command;

use crate::prelude::*;

/// Allow read-only `gh` operations, ask for write operations, and handle bot contexts.
pub fn gh_rules() -> Vec<SimpleRule> {
    vec![
        gh_run_list(),
        gh_run_view(),
        gh_release_list(),
        gh_pr_comment__bot(),
        gh_pr_comment(),
        gh_api_graphql__mutation(),
        gh_api_graphql__query(),
        gh_api__bot_pr_reply(),
        gh_api__data_flags(),
        gh_api__write_method(),
        gh_api(),
    ]
}

/// Allow `gh run list`.
fn gh_run_list() -> SimpleRule {
    SimpleRule::new(
        "gh_run_list",
        "gh run list",
        Outcome::allow("Read-only gh run list"),
    )
}

/// Allow `gh run view`.
fn gh_run_view() -> SimpleRule {
    SimpleRule::new(
        "gh_run_view",
        "gh run view",
        Outcome::allow("Read-only gh run view"),
    )
}

/// Allow `gh release list`.
fn gh_release_list() -> SimpleRule {
    SimpleRule::new(
        "gh_release_list",
        "gh release list",
        Outcome::allow("Read-only gh release list"),
    )
}

/// Allow bot PR comment (`StudioLE`).
fn gh_pr_comment__bot() -> SimpleRule {
    SimpleRule {
        id: "gh_pr_comment__bot".to_owned(),
        prefix: "gh pr".to_owned(),
        condition: Some(is_bot_pr_comment),
        outcome: Outcome::allow("PR comment"),
        ..Default::default()
    }
}

/// Ask for PR comment.
fn gh_pr_comment() -> SimpleRule {
    SimpleRule {
        id: "gh_pr_comment".to_owned(),
        prefix: "gh pr".to_owned(),
        condition: Some(|cmd, _, _| is_pr_comment(cmd)),
        outcome: Outcome::ask("PR comment requires approval"),
        ..Default::default()
    }
}

/// Ask for GraphQL mutation.
fn gh_api_graphql__mutation() -> SimpleRule {
    SimpleRule {
        id: "gh_api_graphql__mutation".to_owned(),
        prefix: "gh api".to_owned(),
        condition: Some(is_graphql_mutation),
        outcome: Outcome::ask("GitHub GraphQL mutation"),
        ..Default::default()
    }
}

/// Allow GraphQL query.
fn gh_api_graphql__query() -> SimpleRule {
    SimpleRule {
        id: "gh_api_graphql__query".to_owned(),
        prefix: "gh api".to_owned(),
        condition: Some(is_graphql_query),
        outcome: Outcome::allow("Read-only GraphQL query"),
        ..Default::default()
    }
}

/// Allow bot PR reply.
fn gh_api__bot_pr_reply() -> SimpleRule {
    SimpleRule {
        id: "gh_api__bot_pr_reply".to_owned(),
        prefix: "gh api".to_owned(),
        condition: Some(is_bot_pr_reply),
        outcome: Outcome::allow("PR comment reply"),
        ..Default::default()
    }
}

/// Ask for API with data flags.
fn gh_api__data_flags() -> SimpleRule {
    SimpleRule {
        id: "gh_api__data_flags".to_owned(),
        prefix: "gh api".to_owned(),
        condition: Some(|cmd, _, _| has_data_flags(cmd)),
        outcome: Outcome::ask("GitHub API request with data flags"),
        ..Default::default()
    }
}

/// Ask for API write method.
fn gh_api__write_method() -> SimpleRule {
    SimpleRule {
        id: "gh_api__write_method".to_owned(),
        prefix: "gh api".to_owned(),
        condition: Some(has_write_method),
        outcome: Outcome::ask("GitHub API write method"),
        ..Default::default()
    }
}

/// Allow read-only `gh api`.
fn gh_api() -> SimpleRule {
    SimpleRule::new(
        "gh_api",
        "gh api",
        Outcome::allow("Read-only gh api command"),
    )
}

fn is_pr_comment(cmd: &SimpleContext) -> bool {
    cmd.args.first().is_some_and(|a| a == "pr") && cmd.args.get(1).is_some_and(|a| a == "comment")
}

fn is_bot_pr_comment(
    cmd: &SimpleContext,
    _complete: &CompleteContext,
    settings: &Settings,
) -> bool {
    if !is_pr_comment(cmd) {
        return false;
    }
    let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
    let gh_user = get_gh_user().unwrap_or_default();
    let repo = extract_repo_flag(&args).unwrap_or_else(|| get_current_repo().unwrap_or_default());
    let org_prefix = format!("{}/", settings.bot_org);
    repo.starts_with(&org_prefix) && gh_user == settings.bot_username
}

fn is_graphql_query(
    cmd: &SimpleContext,
    _complete: &CompleteContext,
    _settings: &Settings,
) -> bool {
    cmd.args.get(1).is_some_and(|a| a == "graphql")
}

fn is_graphql_mutation(
    cmd: &SimpleContext,
    _complete: &CompleteContext,
    _settings: &Settings,
) -> bool {
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

fn is_bot_pr_reply(cmd: &SimpleContext, _complete: &CompleteContext, settings: &Settings) -> bool {
    if !has_data_flags(cmd) {
        return false;
    }
    let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
    let gh_user = get_gh_user().unwrap_or_default();
    let repo = extract_api_repo(&args).unwrap_or_default();
    let org_prefix = format!("{}/", settings.bot_org);
    has_pr_reply_path(&args) && repo.starts_with(&org_prefix) && gh_user == settings.bot_username
}

fn has_write_method(
    cmd: &SimpleContext,
    _complete: &CompleteContext,
    _settings: &Settings,
) -> bool {
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
            return args.get(i + 1).map(|s| unquote_str(s));
        }
    }
    None
}

fn extract_api_repo(args: &[&str]) -> Option<String> {
    for arg in args {
        let unquoted = unquote_str(arg);
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
        let unquoted = unquote_str(arg);
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
    fn _non_gh() {
        // ls and echo are Allow via safe_rules
        let outcome = evaluate_expect_outcome("ls -la");
        assert_eq!(outcome.decision, Decision::Allow);
        let outcome = evaluate_expect_outcome("echo hello");
        assert_eq!(outcome.decision, Decision::Allow);
        // git status is Allow via git_approval
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_non_api() {
        let reason = evaluate_expect_skip("gh pr list");
        assert_eq!(reason, SkipReason::NoMatches);
        let reason = evaluate_expect_skip("gh issue view 123");
        assert_eq!(reason, SkipReason::NoMatches);
        let reason = evaluate_expect_skip("gh repo view");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _gh_run_list() {
        let outcome = evaluate_expect_outcome("gh run list");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_run_list__flags() {
        let outcome = evaluate_expect_outcome("gh run list --limit 10");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_run_view() {
        let outcome = evaluate_expect_outcome("gh run view 12345");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_run_view__log() {
        let outcome = evaluate_expect_outcome("gh run view 12345 --log");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_release_list() {
        let outcome = evaluate_expect_outcome("gh release list");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_release_list__flags() {
        let outcome = evaluate_expect_outcome("gh release list --limit 10");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api() {
        let outcome = evaluate_expect_outcome("gh api user");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__repos() {
        let outcome = evaluate_expect_outcome("gh api repos/owner/repo");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__pulls() {
        let outcome = evaluate_expect_outcome("gh api repos/owner/repo/pulls");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__write_method__post() {
        let outcome = evaluate_expect_outcome("gh api -X POST /repos/owner/repo/issues");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__write_method__put() {
        let outcome = evaluate_expect_outcome("gh api -X PUT /repos/owner/repo/issues/1");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__write_method__patch() {
        let outcome = evaluate_expect_outcome("gh api -X PATCH /repos/owner/repo/issues/1");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__write_method__delete() {
        let outcome = evaluate_expect_outcome("gh api -X DELETE /repos/owner/repo/issues/1");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__pipe_base64() {
        let outcome =
            evaluate_expect_outcome("gh api repos/USER/REPO/readme --jq .content 2>&1 | base64 -d");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__pipe_jq() {
        let outcome = evaluate_expect_outcome("gh api repos/owner/repo/pulls | jq -r '.[].title'");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__jq_pipe() {
        let outcome =
            evaluate_expect_outcome("gh api repos/owner/repo/readme --jq '.content | @base64d'");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__d_before_jq() {
        let outcome = evaluate_expect_outcome(
            "gh api repos/owner/repo -d @body.json --jq '.content | @base64d'",
        );
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api_graphql__query() {
        let outcome = evaluate_expect_outcome(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussions(first: 10) { nodes { title } } } }'",
        );
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api_graphql__query_jq() {
        let outcome = evaluate_expect_outcome(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussion(number: 97) { author { login } } } }' --jq '.data.repository.discussion'",
        );
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api_graphql__query_explicit() {
        let outcome =
            evaluate_expect_outcome("gh api graphql -f query='query { viewer { login } }'");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api_graphql__mutation() {
        let outcome = evaluate_expect_outcome(
            "gh api graphql -f query='mutation { addComment(input: {subjectId: \"123\", body: \"test\"}) { commentEdge { node { body } } } }'",
        );
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__f() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo/issues -f title=test");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__cap_f() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo/issues -F body=test");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__field() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo/issues --field title=test");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__d() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo -d @body.json");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__data() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo --data @body.json");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_api__data_flags__input() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo --input file.json");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_pr_comment() {
        let outcome = evaluate_expect_outcome("gh pr comment 123 --body 'test'");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _gh_pr_comment__bot_studiole() {
        let parsed = Parser::new()
            .parse_str("gh pr comment 123 --body 'test'")
            .expect("should not error")
            .expect("should parse");
        let cmd = parsed.all_commands().next().expect("should have command");
        let settings = Settings::mock();
        let is_bot = super::is_bot_pr_comment(cmd, &parsed, &settings);
        let is_comment = super::is_pr_comment(cmd);
        assert!(is_comment);
        // is_bot depends on live gh auth - just verify the function runs
        let _ = is_bot;
    }

    #[test]
    fn _gh_pr_comment__bot_repo_flag() {
        let parsed = Parser::new()
            .parse_str("gh pr comment 123 --body 'test' -R StudioLE/some-repo")
            .expect("should not error")
            .expect("should parse");
        let cmd = parsed.all_commands().next().expect("should have command");
        assert!(super::is_pr_comment(cmd));
        let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
        let repo = super::extract_repo_flag(&args);
        assert_eq!(repo, Some("StudioLE/some-repo".to_owned()));
    }

    #[test]
    fn _gh_api__bot_pr_reply() {
        let parsed = Parser::new().parse_str(
            "gh api repos/StudioLE/some-repo/pulls/1/comments/2/replies -d '{\"body\":\"test\"}'",
        )
        .expect("should not error")
        .expect("should parse");
        let cmd = parsed.all_commands().next().expect("should have command");
        assert!(super::has_data_flags(cmd));
        let args: Vec<&str> = cmd.args.iter().map(String::as_str).collect();
        assert!(super::has_pr_reply_path(&args));
        let repo = super::extract_api_repo(&args);
        assert_eq!(repo, Some("StudioLE/some-repo".to_owned()));
    }
}
