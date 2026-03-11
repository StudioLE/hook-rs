//! Rules for GitHub CLI commands, distinguishing read vs write operations.

use crate::prelude::*;

/// Allow read-only `gh` operations, ask for write operations, and handle bot contexts.
pub fn gh_rules() -> Vec<SimpleRule> {
    vec![
        gh_run_list(),
        gh_run_view(),
        gh_release_list(),
        gh_pr_comment(),
        gh_api_graphql__mutation(),
        gh_api_graphql__query(),
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

/// Ask for PR comment.
fn gh_pr_comment() -> SimpleRule {
    SimpleRule::new(
        "gh_pr_comment",
        "gh pr comment",
        Outcome::ask("PR comment requires approval"),
    )
}

/// Ask for GraphQL mutation.
fn gh_api_graphql__mutation() -> SimpleRule {
    SimpleRule {
        id: "gh_api_graphql__mutation".to_owned(),
        prefix: "gh api graphql".to_owned(),
        condition: Some(has_mutation_in_args),
        outcome: Outcome::ask("GitHub GraphQL mutation"),
        ..Default::default()
    }
}

/// Allow GraphQL query.
fn gh_api_graphql__query() -> SimpleRule {
    SimpleRule::new(
        "gh_api_graphql__query",
        "gh api graphql",
        Outcome::allow("Read-only GraphQL query"),
    )
}

/// Ask for API with data flags.
fn gh_api__data_flags() -> SimpleRule {
    SimpleRule {
        id: "gh_api__data_flags".to_owned(),
        prefix: "gh api".to_owned(),
        with_any: Some(vec![
            "-d".to_owned(),
            "--data".to_owned(),
            "-f".to_owned(),
            "--field".to_owned(),
            "-F".to_owned(),
            "--raw-field".to_owned(),
            "--input".to_owned(),
        ]),
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

fn has_mutation_in_args(
    cmd: &SimpleContext,
    _complete: &CompleteContext,
    _settings: &Settings,
) -> bool {
    cmd.args
        .iter()
        .any(|a| a.to_lowercase().contains("mutation"))
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
}
