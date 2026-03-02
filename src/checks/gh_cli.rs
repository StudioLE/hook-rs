use regex::Regex;
use std::process::Command;
use std::sync::LazyLock;

use crate::prelude::*;

static GH_CMD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^gh\s").expect("valid regex"));

static GH_PR_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^gh\s+pr\s+comment").expect("valid regex"));

static GH_RUN_READ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^gh\s+run\s+(?:list|view)\b").expect("valid regex"));

static GH_RELEASE_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^gh\s+release\s+list\b").expect("valid regex"));

static GH_API: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^gh\s+api\s").expect("valid regex"));

static GH_GRAPHQL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^gh\s+api\s+graphql\b").expect("valid regex"));

static MUTATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bmutation\b").expect("valid regex"));

static DATA_FLAGS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:-d\s|--data\s|--data=|-f\s|--field\s|--field=|-F\s|--raw-field\s|--input\s|--input=)",
    )
    .expect("valid regex")
});

static WRITE_METHOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)-X\s*(POST|PUT|PATCH|DELETE)").expect("valid regex"));

static PR_COMMENT_REPLY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"pulls/[0-9]+/comments/[0-9]+/replies").expect("valid regex"));

static REPO_FLAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:-R|--repo)\s+(\S+)").expect("valid regex"));

static API_REPO_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"repos/([^/]+/[^/]+)/").expect("valid regex"));

/// Strip shell pipes to avoid false positives (e.g. `| base64 -d`).
/// Matches ` | <command>` but not jq pipes like `| @base64d` or `| .field`.
fn strip_shell_pipes(command: &str) -> &str {
    if let Some(pos) = command.find(" | ") {
        let after = &command[pos + 3..];
        if after.starts_with(|c: char| c.is_ascii_lowercase()) {
            return &command[..pos];
        }
    }
    command
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

#[must_use]
pub fn check(command: &str) -> Option<CheckResult> {
    let gh_user = get_gh_user().unwrap_or_default();
    let current_repo = get_current_repo().unwrap_or_default();
    check_with_context(command, &gh_user, &current_repo)
}

#[must_use]
pub fn check_with_context(command: &str, gh_user: &str, current_repo: &str) -> Option<CheckResult> {
    if !GH_CMD.is_match(command) {
        return None;
    }
    if GH_PR_COMMENT.is_match(command) {
        let repo = REPO_FLAG
            .captures(command)
            .map_or_else(|| current_repo.to_owned(), |c| c[1].to_owned());
        if repo.starts_with("StudioLE/") && gh_user == "StudioLE-Bot" {
            return Some(CheckResult::allow("PR comment"));
        }
        return Some(CheckResult::ask(format!(
            "PR comment as {gh_user} in {repo}"
        )));
    }
    if GH_RUN_READ.is_match(command) {
        return Some(CheckResult::allow("Read-only gh run command"));
    }
    if GH_RELEASE_LIST.is_match(command) {
        return Some(CheckResult::allow("Read-only gh release list command"));
    }
    if !GH_API.is_match(command) {
        return None;
    }
    let gh_api_cmd = strip_shell_pipes(command);
    if GH_GRAPHQL.is_match(command) {
        if MUTATION.is_match(command) {
            return Some(CheckResult::ask("GitHub GraphQL mutation"));
        }
        return Some(CheckResult::allow("Read-only GraphQL query"));
    }
    if DATA_FLAGS.is_match(gh_api_cmd) {
        let repo = API_REPO_PATH
            .captures(command)
            .map(|c| c[1].to_owned())
            .unwrap_or_default();
        if PR_COMMENT_REPLY.is_match(command)
            && repo.starts_with("StudioLE/")
            && gh_user == "StudioLE-Bot"
        {
            return Some(CheckResult::allow("PR comment reply"));
        }
        return Some(CheckResult::ask(format!(
            "GitHub API requests with data flags as {gh_user} in {repo}"
        )));
    }
    if let Some(caps) = WRITE_METHOD.captures(gh_api_cmd) {
        let method = caps[1].to_uppercase();
        return Some(CheckResult::ask(format!("GitHub API {method}")));
    }
    Some(CheckResult::allow("Read-only gh api command"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    fn check_t(command: &str) -> Option<CheckResult> {
        check_with_context(command, "test-user", "owner/repo")
    }

    fn check_bot(command: &str) -> Option<CheckResult> {
        check_with_context(command, "StudioLE-Bot", "StudioLE/some-repo")
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
            "gh pr comment 123 --body 'test' -R StudioLE/some-repo",
            "StudioLE-Bot",
            "other/repo"
        ));
    }

    #[test]
    fn gh_api_pr_reply_bot_studiole() {
        assert_yaml_snapshot!(check_with_context(
            "gh api repos/StudioLE/some-repo/pulls/1/comments/2/replies -d '{\"body\":\"test\"}'",
            "StudioLE-Bot",
            ""
        ));
    }
}
