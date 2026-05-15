//! Rules for GitHub CLI commands, distinguishing read vs write operations.

use crate::prelude::*;

/// Allow read-only `gh` operations, ask for write operations, and handle bot contexts.
pub fn gh_rules() -> Vec<BashRule> {
    vec![
        gh_run_list(),
        gh_run_view(),
        gh_release_list(),
        gh_pr_view(),
        gh_search_code(),
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
        Outcome::allow("Read-only `gh run list`"),
    )
}

/// Allow `gh run view`.
fn gh_run_view() -> BashRule {
    BashRule::new(
        "gh_run_view",
        "gh run view",
        Outcome::allow("Read-only `gh run view`"),
    )
}

/// Allow `gh release list`.
fn gh_release_list() -> BashRule {
    BashRule::new(
        "gh_release_list",
        "gh release list",
        Outcome::allow("Read-only `gh release list`"),
    )
}

/// Allow `gh pr view`.
fn gh_pr_view() -> BashRule {
    BashRule::new(
        "gh_pr_view",
        "gh pr view",
        Outcome::allow("Read-only `gh pr view`"),
    )
}

/// Allow `gh search code`.
fn gh_search_code() -> BashRule {
    BashRule::new(
        "gh_search_code",
        "gh search code",
        Outcome::allow("Read-only `gh search code`"),
    )
}

/// Ask for PR comment.
fn gh_pr_comment() -> BashRule {
    BashRule::new(
        "gh_pr_comment",
        "gh pr comment",
        Outcome::ask("`gh pr comment` requires approval"),
    )
}

/// Ask for GraphQL mutation.
fn gh_api_graphql__mutation() -> BashRule {
    BashRule {
        id: "gh_api_graphql__mutation".to_owned(),
        command: "gh api graphql".to_owned(),
        with_any: Some(vec![ArgMatcher::new("*mutation*")]),
        outcome: Outcome::ask("`gh api graphql` with mutation requires approval"),
        ..Default::default()
    }
}

/// Allow GraphQL query (no mutation).
fn gh_api_graphql__query() -> BashRule {
    BashRule {
        id: "gh_api_graphql__query".to_owned(),
        command: "gh api graphql".to_owned(),
        without_any: Some(vec![ArgMatcher::new("*mutation*")]),
        outcome: Outcome::allow("Read-only `gh api graphql` query"),
        ..Default::default()
    }
}

/// Ask for API with data flags.
///
/// Excludes `gh api graphql` which is handled by dedicated graphql rules.
fn gh_api__data_flags() -> BashRule {
    BashRule {
        id: "gh_api__data_flags".to_owned(),
        command: "gh api".to_owned(),
        with_any: Some(vec![
            ArgMatcher::new("-d"),
            ArgMatcher::new("--data"),
            ArgMatcher::new("-f"),
            ArgMatcher::new("--field"),
            ArgMatcher::new("-F"),
            ArgMatcher::new("--raw-field"),
            ArgMatcher::new("--input"),
        ]),
        without_any: Some(vec![ArgMatcher::new("graphql")]),
        outcome: Outcome::ask("`gh api` with data flags requires approval"),
        ..Default::default()
    }
}

/// Ask for API write method.
///
/// Excludes `gh api graphql` which is handled by dedicated graphql rules.
fn gh_api__write_method() -> BashRule {
    BashRule {
        id: "gh_api__write_method".to_owned(),
        command: "gh api".to_owned(),
        with_any: Some(vec![
            ArgMatcher::new("-X").ivalue("{POST,PUT,PATCH,DELETE}"),
        ]),
        without_any: Some(vec![ArgMatcher::new("graphql")]),
        outcome: Outcome::ask("`gh api` with write method requires approval"),
        ..Default::default()
    }
}

/// Allow read-only `gh api` (no data flags or write methods).
///
/// Excludes `gh api graphql` which is handled by dedicated graphql rules.
fn gh_api__read_only() -> BashRule {
    BashRule {
        id: "gh_api__read_only".to_owned(),
        command: "gh api".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("-d"),
            ArgMatcher::new("--data"),
            ArgMatcher::new("-f"),
            ArgMatcher::new("--field"),
            ArgMatcher::new("-F"),
            ArgMatcher::new("--raw-field"),
            ArgMatcher::new("--input"),
            ArgMatcher::new("-X").ivalue("{POST,PUT,PATCH,DELETE}"),
            ArgMatcher::new("graphql"),
        ]),
        outcome: Outcome::allow("Read-only `gh api`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn gh_non_api() {
        let result = eval_rules(gh_rules(), "gh pr list");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
        let result = eval_rules(gh_rules(), "gh issue view 123");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
        let result = eval_rules(gh_rules(), "gh repo view");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn gh_run_list() {
        let result = eval_rules(gh_rules(), "gh run list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_run_list_flags() {
        let result = eval_rules(gh_rules(), "gh run list --limit 10");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_run_view() {
        let result = eval_rules(gh_rules(), "gh run view 12345");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_run_view_log() {
        let result = eval_rules(gh_rules(), "gh run view 12345 --log");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_release_list() {
        let result = eval_rules(gh_rules(), "gh release list");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_release_list_flags() {
        let result = eval_rules(gh_rules(), "gh release list --limit 10");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api() {
        let result = eval_rules(gh_rules(), "gh api user");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_repos() {
        let result = eval_rules(gh_rules(), "gh api repos/owner/repo");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_pulls() {
        let result = eval_rules(gh_rules(), "gh api repos/owner/repo/pulls");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_write_method_post() {
        let result = eval_rules(gh_rules(), "gh api -X POST /repos/owner/repo/issues");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_write_method_put() {
        let result = eval_rules(gh_rules(), "gh api -X PUT /repos/owner/repo/issues/1");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_write_method_patch() {
        let result = eval_rules(gh_rules(), "gh api -X PATCH /repos/owner/repo/issues/1");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_write_method_delete() {
        let result = eval_rules(gh_rules(), "gh api -X DELETE /repos/owner/repo/issues/1");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_pipe_base64() {
        let result = eval_rules(
            gh_rules(),
            "gh api repos/USER/REPO/readme --jq .content 2>&1 | base64 -d",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn gh_api_pipe_jq() {
        let result = eval_rules(
            gh_rules(),
            "gh api repos/owner/repo/pulls | jq -r '.[].title'",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn gh_api_jq_pipe() {
        let result = eval_rules(
            gh_rules(),
            "gh api repos/owner/repo/readme --jq '.content | @base64d'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_data_flags_d_before_jq() {
        let result = eval_rules(
            gh_rules(),
            "gh api repos/owner/repo -d @body.json --jq '.content | @base64d'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_graphql_query() {
        let result = eval_rules(
            gh_rules(),
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussions(first: 10) { nodes { title } } } }'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_graphql_query_jq() {
        let result = eval_rules(
            gh_rules(),
            "gh api graphql -f query='{ repository(owner: \"owner\", name: \"repo\") { discussion(number: 97) { author { login } } } }' --jq '.data.repository.discussion'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_graphql_query_explicit() {
        let result = eval_rules(
            gh_rules(),
            "gh api graphql -f query='query { viewer { login } }'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_api_graphql_mutation() {
        let result = eval_rules(
            gh_rules(),
            "gh api graphql -f query='mutation { addComment(input: {subjectId: \"123\", body: \"test\"}) { commentEdge { node { body } } } }'",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_data_flags_f() {
        let result = eval_rules(gh_rules(), "gh api /repos/owner/repo/issues -f title=test");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_data_flags_cap_f() {
        let result = eval_rules(gh_rules(), "gh api /repos/owner/repo/issues -F body=test");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_data_flags_field() {
        let result = eval_rules(
            gh_rules(),
            "gh api /repos/owner/repo/issues --field title=test",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_data_flags_d() {
        let result = eval_rules(gh_rules(), "gh api /repos/owner/repo -d @body.json");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_data_flags_data() {
        let result = eval_rules(gh_rules(), "gh api /repos/owner/repo --data @body.json");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_api_data_flags_input() {
        let result = eval_rules(gh_rules(), "gh api /repos/owner/repo --input file.json");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn gh_pr_view() {
        let result = eval_rules(gh_rules(), "gh pr view 228");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_pr_view_json_fields() {
        let result = eval_rules(gh_rules(), "gh pr view 228 --json reviews,commits");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_search_code() {
        let result = eval_rules(
            gh_rules(),
            "gh search code --repo rust-lang/rust-analyzer \"fn diagnostics\" --limit 5",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn gh_pr_comment() {
        let result = eval_rules(gh_rules(), "gh pr comment 123 --body 'test'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Ask);
    }
}
