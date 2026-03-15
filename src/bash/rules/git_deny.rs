//! Deny rules for destructive git operations.

use crate::prelude::*;

/// Deny destructive git operations.
pub fn git_deny_rules() -> Vec<SimpleRule> {
    vec![
        git_reset_hard(),
        git_stash_pop(),
        git_stash_drop(),
        git_stash_clear(),
        git_clean_d(),
        git_checkout_discard(),
    ]
}

/// Deny `git reset --hard`.
fn git_reset_hard() -> SimpleRule {
    SimpleRule {
        id: "git_reset_hard".to_owned(),
        prefix: "git reset".to_owned(),
        with_any: Some(vec![Arg::new("--hard")]),
        outcome: Outcome::deny("git reset --hard discards uncommitted changes"),
        ..Default::default()
    }
}

/// Deny `git stash pop`.
fn git_stash_pop() -> SimpleRule {
    SimpleRule {
        id: "git_stash_pop".to_owned(),
        prefix: "git stash pop".to_owned(),
        outcome: Outcome::deny("git stash pop can cause merge conflicts and lose stash"),
        ..Default::default()
    }
}

/// Deny `git stash drop`.
fn git_stash_drop() -> SimpleRule {
    SimpleRule {
        id: "git_stash_drop".to_owned(),
        prefix: "git stash drop".to_owned(),
        outcome: Outcome::deny("git stash drop permanently deletes a stash entry"),
        ..Default::default()
    }
}

/// Deny `git stash clear`.
fn git_stash_clear() -> SimpleRule {
    SimpleRule {
        id: "git_stash_clear".to_owned(),
        prefix: "git stash clear".to_owned(),
        outcome: Outcome::deny("git stash clear permanently deletes all stash entries"),
        ..Default::default()
    }
}

/// Deny `git clean -d`.
fn git_clean_d() -> SimpleRule {
    SimpleRule {
        id: "git_clean_d".to_owned(),
        prefix: "git clean".to_owned(),
        with_any: Some(vec![Arg::new("-d")]),
        outcome: Outcome::deny(
            "git clean with -d is blocked. Use 'git clean -f <file>' for specific files \
             (or -fx if gitignored) or 'git rm -r <dir>' for tracked directories.",
        ),
        ..Default::default()
    }
}

/// Deny `git checkout --`.
fn git_checkout_discard() -> SimpleRule {
    SimpleRule {
        id: "git_checkout_discard".to_owned(),
        prefix: "git checkout".to_owned(),
        with_any: Some(vec![Arg::new("--")]),
        outcome: Outcome::deny(
            "git checkout -- is blocked. Do not discard changes to revert your mistakes. \
             Fix the code properly.",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use insta::assert_yaml_snapshot;

    // === git reset --hard tests ===

    #[test]
    fn _git_reset_hard() {
        let outcome = evaluate_expect_outcome("git reset --hard");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_reset_hard__head() {
        let outcome = evaluate_expect_outcome("git reset --hard HEAD");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_reset_hard__head_1() {
        let outcome = evaluate_expect_outcome("git reset --hard HEAD~1");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_reset_hard__origin_main() {
        let outcome = evaluate_expect_outcome("git reset --hard origin/main");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_reset_hard__chained() {
        let outcome = evaluate_expect_outcome("git fetch && git reset --hard origin/main");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_reset_hard__in_chain() {
        let outcome = evaluate_expect_outcome("git stash && git reset --hard && git stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_reset() {
        let reason = evaluate_expect_skip("git reset");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_reset__head() {
        let reason = evaluate_expect_skip("git reset HEAD");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_reset__soft() {
        let reason = evaluate_expect_skip("git reset --soft HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_reset__mixed() {
        let reason = evaluate_expect_skip("git reset --mixed HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_reset__file() {
        let reason = evaluate_expect_skip("git reset HEAD -- file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_status() {
        let outcome = evaluate_expect_outcome("git status");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _echo_reset_hard() {
        let outcome = evaluate_expect_outcome("echo git reset --hard is dangerous");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _grep_reset_hard() {
        let outcome = evaluate_expect_outcome("grep 'git reset --hard' README.md");
        assert_yaml_snapshot!(outcome);
    }

    // === git stash tests ===

    #[test]
    fn _git_stash_pop() {
        let outcome = evaluate_expect_outcome("git stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_pop__ref() {
        let outcome = evaluate_expect_outcome("git stash pop stash@{0}");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_pop__index() {
        let outcome = evaluate_expect_outcome("git stash pop --index");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_pop__chained() {
        let outcome = evaluate_expect_outcome("git stash && git pull && git stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_drop() {
        let outcome = evaluate_expect_outcome("git stash drop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_drop__ref() {
        let outcome = evaluate_expect_outcome("git stash drop stash@{0}");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_drop__stash_2() {
        let outcome = evaluate_expect_outcome("git stash drop stash@{2}");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_drop__chained() {
        let outcome = evaluate_expect_outcome("git stash list && git stash drop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_clear() {
        let outcome = evaluate_expect_outcome("git stash clear");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash_clear__chained() {
        let outcome = evaluate_expect_outcome("false || git stash clear");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_stash() {
        let reason = evaluate_expect_skip("git stash");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__push() {
        let reason = evaluate_expect_skip("git stash push");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__push_m() {
        let reason = evaluate_expect_skip("git stash push -m 'wip'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__apply() {
        let reason = evaluate_expect_skip("git stash apply");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__apply_ref() {
        let reason = evaluate_expect_skip("git stash apply stash@{0}");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__list() {
        let reason = evaluate_expect_skip("git stash list");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__show() {
        let reason = evaluate_expect_skip("git stash show");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__show_p() {
        let reason = evaluate_expect_skip("git stash show -p");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_stash__branch() {
        let reason = evaluate_expect_skip("git stash branch newbranch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _echo_stash_pop() {
        let outcome = evaluate_expect_outcome("echo git stash pop is blocked");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _grep_stash_drop() {
        let outcome = evaluate_expect_outcome("grep 'git stash drop' file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _cat_stash_clear() {
        let outcome = evaluate_expect_outcome("cat stash-clear-notes.txt");
        assert_yaml_snapshot!(outcome);
    }

    // === git clean tests ===

    #[test]
    fn _git_clean_d__fd() {
        let outcome = evaluate_expect_outcome("git clean -fd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_clean_d__fxd() {
        let outcome = evaluate_expect_outcome("git clean -fxd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_clean_d() {
        let outcome = evaluate_expect_outcome("git clean -d");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_clean_d__df() {
        let outcome = evaluate_expect_outcome("git clean -df");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_clean_d__dxf() {
        let outcome = evaluate_expect_outcome("git clean -dxf");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_clean_d__chained() {
        let outcome = evaluate_expect_outcome("ls && git clean -fd");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_clean__f() {
        let reason = evaluate_expect_skip("git clean -f file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_clean__fx() {
        let reason = evaluate_expect_skip("git clean -fx file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_clean__fx_dash_filename() {
        let reason = evaluate_expect_skip("git clean -fx /path/to/some-dash-delimited-file.sh");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_clean__f_dash_path() {
        let reason = evaluate_expect_skip("git clean -f /path/dir-name/file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_clean__n() {
        let reason = evaluate_expect_skip("git clean -n");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _echo_git_clean() {
        let outcome = evaluate_expect_outcome("echo git clean -fxd");
        assert_yaml_snapshot!(outcome);
    }

    // === git checkout -- tests ===

    #[test]
    fn _git_checkout_discard__head_file() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__head_dot() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- .");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__head_src() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- src/");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__head_multiple() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- file1.txt file2.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__chained_head() {
        let outcome = evaluate_expect_outcome("git status && git checkout HEAD -- file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__head_in_chain() {
        let outcome =
            evaluate_expect_outcome("git stash && git checkout HEAD -- . && git stash pop");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__file() {
        let outcome = evaluate_expect_outcome("git checkout -- file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__dot() {
        let outcome = evaluate_expect_outcome("git checkout -- .");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__src() {
        let outcome = evaluate_expect_outcome("git checkout -- src/");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout_discard__chained() {
        let outcome = evaluate_expect_outcome("git status && git checkout -- file.txt");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _git_checkout__branch() {
        let reason = evaluate_expect_skip("git checkout main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_checkout__b() {
        let reason = evaluate_expect_skip("git checkout -b new-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_checkout__head_1() {
        let reason = evaluate_expect_skip("git checkout HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _git_checkout__head_caret() {
        let reason = evaluate_expect_skip("git checkout HEAD^");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _echo_checkout_head() {
        let outcome = evaluate_expect_outcome("echo git checkout HEAD -- is dangerous");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _echo_checkout_discard() {
        let outcome = evaluate_expect_outcome("echo git checkout -- is dangerous");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _grep_checkout_head() {
        let outcome = evaluate_expect_outcome("grep 'git checkout HEAD --' README.md");
        assert_yaml_snapshot!(outcome);
    }

    #[test]
    fn _grep_checkout_discard() {
        let outcome = evaluate_expect_outcome("grep 'git checkout --' README.md");
        assert_yaml_snapshot!(outcome);
    }
}
