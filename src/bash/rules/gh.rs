//! Rules for GitHub CLI commands, distinguishing read vs write operations.

use crate::prelude::*;

/// Allow read-only `gh` operations, ask for write operations, and handle bot contexts.
pub fn gh_rules() -> Vec<BashRule> {
    vec![
        gh_run_list(),
        gh_run_view(),
        gh_release_list(),
        gh_pr_comment(),
        gh_api_graphql__mutation(),
        gh_api_graphql__query(),
        gh_api__data_flags(),
        gh_api__write_method(),
        gh_api__read_only(),
    ]
}

/// Allow `gh run list`.
fn gh_run_list() -> BashRule {
    BashRule::new(
        "gh_run_list",
        "gh run list",
        Outcome::allow("Read-only gh run list"),
    )
}

/// Allow `gh run view`.
fn gh_run_view() -> BashRule {
    BashRule::new(
        "gh_run_view",
        "gh run view",
        Outcome::allow("Read-only gh run view"),
    )
}

/// Allow `gh release list`.
fn gh_release_list() -> BashRule {
    BashRule::new(
        "gh_release_list",
        "gh release list",
        Outcome::allow("Read-only gh release list"),
    )
}

/// Ask for PR comment.
fn gh_pr_comment() -> BashRule {
    BashRule::new(
        "gh_pr_comment",
        "gh pr comment",
        Outcome::ask("PR comment requires approval"),
    )
}

/// Ask for GraphQL mutation.
fn gh_api_graphql__mutation() -> BashRule {
    BashRule {
        id: "gh_api_graphql__mutation".to_owned(),
        command: "gh api graphql".to_owned(),
        with_any: Some(vec![Arg::new("*mutation*")]),
        outcome: Outcome::ask("GitHub GraphQL mutation"),
        ..Default::default()
    }
}

/// Allow GraphQL query (no mutation).
fn gh_api_graphql__query() -> BashRule {
    BashRule {
        id: "gh_api_graphql__query".to_owned(),
        command: "gh api graphql".to_owned(),
        without_any: Some(vec![Arg::new("*mutation*")]),
        outcome: Outcome::allow("Read-only GraphQL query"),
        ..Default::default()
    }
}

/// Ask for API with data flags.
fn gh_api__data_flags() -> BashRule {
    BashRule {
        id: "gh_api__data_flags".to_owned(),
        command: "gh api".to_owned(),
        with_any: Some(vec![
            Arg::new("-d"),
            Arg::new("--data"),
            Arg::new("-f"),
            Arg::new("--field"),
            Arg::new("-F"),
            Arg::new("--raw-field"),
            Arg::new("--input"),
        ]),
        outcome: Outcome::ask("GitHub API request with data flags"),
        ..Default::default()
    }
}

/// Ask for API write method.
fn gh_api__write_method() -> BashRule {
    BashRule {
        id: "gh_api__write_method".to_owned(),
        command: "gh api".to_owned(),
        with_any: Some(vec![Arg::new("-X").ivalue("{POST,PUT,PATCH,DELETE}")]),
        outcome: Outcome::ask("GitHub API write method"),
        ..Default::default()
    }
}

/// Allow read-only `gh api` (no data flags or write methods).
fn gh_api__read_only() -> BashRule {
    BashRule {
        id: "gh_api__read_only".to_owned(),
        command: "gh api".to_owned(),
        without_any: Some(vec![
            Arg::new("-d"),
            Arg::new("--data"),
            Arg::new("-f"),
            Arg::new("--field"),
            Arg::new("-F"),
            Arg::new("--raw-field"),
            Arg::new("--input"),
            Arg::new("-X").ivalue("{POST,PUT,PATCH,DELETE}"),
        ]),
        outcome: Outcome::allow("Read-only gh api command"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
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
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_run_list__flags() {
        let outcome = evaluate_expect_outcome("gh run list --limit 10");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_run_view() {
        let outcome = evaluate_expect_outcome("gh run view 12345");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_run_view__log() {
        let outcome = evaluate_expect_outcome("gh run view 12345 --log");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_release_list() {
        let outcome = evaluate_expect_outcome("gh release list");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_release_list__flags() {
        let outcome = evaluate_expect_outcome("gh release list --limit 10");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api() {
        let outcome = evaluate_expect_outcome("gh api user");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api__repos() {
        let outcome = evaluate_expect_outcome("gh api repos/owner/repo");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api__pulls() {
        let outcome = evaluate_expect_outcome("gh api repos/owner/repo/pulls");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api__write_method__post() {
        let outcome = evaluate_expect_outcome("gh api -X POST /repos/owner/repo/issues");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__write_method__put() {
        let outcome = evaluate_expect_outcome("gh api -X PUT /repos/owner/repo/issues/1");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__write_method__patch() {
        let outcome = evaluate_expect_outcome("gh api -X PATCH /repos/owner/repo/issues/1");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__write_method__delete() {
        let outcome = evaluate_expect_outcome("gh api -X DELETE /repos/owner/repo/issues/1");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__pipe_base64() {
        let outcome =
            evaluate_expect_outcome("gh api repos/USER/REPO/readme --jq .content 2>&1 | base64 -d");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api__pipe_jq() {
        let outcome = evaluate_expect_outcome("gh api repos/owner/repo/pulls | jq -r '.[].title'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api__jq_pipe() {
        let outcome =
            evaluate_expect_outcome("gh api repos/owner/repo/readme --jq '.content | @base64d'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _gh_api__data_flags__d_before_jq() {
        let outcome = evaluate_expect_outcome(
            "gh api repos/owner/repo -d @body.json --jq '.content | @base64d'",
        );
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api_graphql__query() {
        let outcome = evaluate_expect_outcome(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussions(first: 10) { nodes { title } } } }'",
        );
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api_graphql__query_jq() {
        let outcome = evaluate_expect_outcome(
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussion(number: 97) { author { login } } } }' --jq '.data.repository.discussion'",
        );
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api_graphql__query_explicit() {
        let outcome =
            evaluate_expect_outcome("gh api graphql -f query='query { viewer { login } }'");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api_graphql__mutation() {
        let outcome = evaluate_expect_outcome(
            "gh api graphql -f query='mutation { addComment(input: {subjectId: \"123\", body: \"test\"}) { commentEdge { node { body } } } }'",
        );
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__data_flags__f() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo/issues -f title=test");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__data_flags__cap_f() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo/issues -F body=test");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__data_flags__field() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo/issues --field title=test");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__data_flags__d() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo -d @body.json");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__data_flags__data() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo --data @body.json");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_api__data_flags__input() {
        let outcome = evaluate_expect_outcome("gh api /repos/owner/repo --input file.json");
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn _gh_pr_comment() {
        let outcome = evaluate_expect_outcome("gh pr comment 123 --body 'test'");
        assert_eq!(outcome.decision, Decision::Ask);
    }
}
