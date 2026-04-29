//! Deny rules for destructive git operations.

use crate::prelude::*;

/// Deny destructive git operations.
pub fn git_deny_rules() -> Vec<BashRule> {
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
fn git_reset_hard() -> BashRule {
    BashRule {
        id: "git_reset_hard".to_owned(),
        command: "git reset".to_owned(),
        with_any: Some(vec![ArgMatcher::new("--hard")]),
        outcome: Outcome::deny("`git reset --hard` is blocked. Discards uncommitted changes"),
        ..Default::default()
    }
}

/// Deny `git stash pop`.
fn git_stash_pop() -> BashRule {
    BashRule {
        id: "git_stash_pop".to_owned(),
        command: "git stash pop".to_owned(),
        outcome: Outcome::deny(
            "`git stash pop` is blocked. Can cause merge conflicts and lose the stash. \
             Alternatives: `git stash apply`",
        ),
        ..Default::default()
    }
}

/// Deny `git stash drop`.
fn git_stash_drop() -> BashRule {
    BashRule {
        id: "git_stash_drop".to_owned(),
        command: "git stash drop".to_owned(),
        outcome: Outcome::deny("`git stash drop` is blocked. Permanently deletes a stash entry"),
        ..Default::default()
    }
}

/// Deny `git stash clear`.
fn git_stash_clear() -> BashRule {
    BashRule {
        id: "git_stash_clear".to_owned(),
        command: "git stash clear".to_owned(),
        outcome: Outcome::deny(
            "`git stash clear` is blocked. Permanently deletes all stash entries",
        ),
        ..Default::default()
    }
}

/// Deny `git clean -d`.
fn git_clean_d() -> BashRule {
    BashRule {
        id: "git_clean_d".to_owned(),
        command: "git clean".to_owned(),
        with_any: Some(vec![ArgMatcher::new("-d")]),
        outcome: Outcome::deny(
            "`git clean -d` is blocked. Alternatives: `git clean -f <file>`, \
             `git clean -fx <file>` (gitignored), `git rm -r <dir>` (tracked)",
        ),
        ..Default::default()
    }
}

/// Deny `git checkout --`.
fn git_checkout_discard() -> BashRule {
    BashRule {
        id: "git_checkout_discard".to_owned(),
        command: "git checkout".to_owned(),
        with_any: Some(vec![ArgMatcher::new("--")]),
        outcome: Outcome::deny(
            "`git checkout --` is blocked. Do not discard changes to revert mistakes; \
             fix the code instead",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn git_reset_hard() {
        let outcome = evaluate_expect_outcome("git reset --hard");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_head() {
        let outcome = evaluate_expect_outcome("git reset --hard HEAD");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_head_1() {
        let outcome = evaluate_expect_outcome("git reset --hard HEAD~1");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_origin_main() {
        let outcome = evaluate_expect_outcome("git reset --hard origin/main");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_chained() {
        let outcome = evaluate_expect_outcome("git fetch && git reset --hard origin/main");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_in_chain() {
        let outcome = evaluate_expect_outcome("git stash && git reset --hard && git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset() {
        let reason = evaluate_expect_skip("git reset");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_head() {
        let reason = evaluate_expect_skip("git reset HEAD");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_soft() {
        let reason = evaluate_expect_skip("git reset --soft HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_mixed() {
        let reason = evaluate_expect_skip("git reset --mixed HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_file() {
        let reason = evaluate_expect_skip("git reset HEAD -- file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_status() {
        let outcome = evaluate_expect_outcome("git status");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_reset_hard() {
        let outcome = evaluate_expect_outcome("echo git reset --hard is dangerous");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_reset_hard() {
        let outcome = evaluate_expect_outcome("rg 'git reset --hard' README.md");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_stash_pop() {
        let outcome = evaluate_expect_outcome("git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop_ref() {
        let outcome = evaluate_expect_outcome("git stash pop stash@{0}");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop_index() {
        let outcome = evaluate_expect_outcome("git stash pop --index");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop_chained() {
        let outcome = evaluate_expect_outcome("git stash && git pull && git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop() {
        let outcome = evaluate_expect_outcome("git stash drop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop_ref() {
        let outcome = evaluate_expect_outcome("git stash drop stash@{0}");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop_stash_2() {
        let outcome = evaluate_expect_outcome("git stash drop stash@{2}");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop_chained() {
        let outcome = evaluate_expect_outcome("git stash list && git stash drop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_clear() {
        let outcome = evaluate_expect_outcome("git stash clear");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_clear_chained() {
        let outcome = evaluate_expect_outcome("false || git stash clear");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash() {
        let reason = evaluate_expect_skip("git stash");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_push() {
        let reason = evaluate_expect_skip("git stash push");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_push_m() {
        let reason = evaluate_expect_skip("git stash push -m 'wip'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_apply() {
        let reason = evaluate_expect_skip("git stash apply");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_apply_ref() {
        let reason = evaluate_expect_skip("git stash apply stash@{0}");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_list() {
        let reason = evaluate_expect_skip("git stash list");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_show() {
        let reason = evaluate_expect_skip("git stash show");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_show_p() {
        let reason = evaluate_expect_skip("git stash show -p");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_branch() {
        let reason = evaluate_expect_skip("git stash branch newbranch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_stash_pop() {
        let outcome = evaluate_expect_outcome("echo git stash pop is blocked");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_stash_drop() {
        let outcome = evaluate_expect_outcome("rg 'git stash drop' file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cat_stash_clear() {
        let outcome = evaluate_expect_outcome("cat stash-clear-notes.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_clean_d_fd() {
        let outcome = evaluate_expect_outcome("git clean -fd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_fxd() {
        let outcome = evaluate_expect_outcome("git clean -fxd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d() {
        let outcome = evaluate_expect_outcome("git clean -d");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_df() {
        let outcome = evaluate_expect_outcome("git clean -df");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_dxf() {
        let outcome = evaluate_expect_outcome("git clean -dxf");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_chained() {
        let outcome = evaluate_expect_outcome("ls && git clean -fd");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_f() {
        let reason = evaluate_expect_skip("git clean -f file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_fx() {
        let reason = evaluate_expect_skip("git clean -fx file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_fx_dash_filename() {
        let reason = evaluate_expect_skip("git clean -fx /path/to/some-dash-delimited-file.sh");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_f_dash_path() {
        let reason = evaluate_expect_skip("git clean -f /path/dir-name/file.txt");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_n() {
        let reason = evaluate_expect_skip("git clean -n");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_git_clean() {
        let outcome = evaluate_expect_outcome("echo git clean -fxd");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn git_checkout_discard_head_file() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_dot() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- .");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_src() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- src/");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_multiple() {
        let outcome = evaluate_expect_outcome("git checkout HEAD -- file1.txt file2.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_chained_head() {
        let outcome = evaluate_expect_outcome("git status && git checkout HEAD -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_in_chain() {
        let outcome =
            evaluate_expect_outcome("git stash && git checkout HEAD -- . && git stash pop");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_file() {
        let outcome = evaluate_expect_outcome("git checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_dot() {
        let outcome = evaluate_expect_outcome("git checkout -- .");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_src() {
        let outcome = evaluate_expect_outcome("git checkout -- src/");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_chained() {
        let outcome = evaluate_expect_outcome("git status && git checkout -- file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_branch() {
        let reason = evaluate_expect_skip("git checkout main");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_b() {
        let reason = evaluate_expect_skip("git checkout -b new-branch");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_head_1() {
        let reason = evaluate_expect_skip("git checkout HEAD~1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_head_caret() {
        let reason = evaluate_expect_skip("git checkout HEAD^");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_checkout_head() {
        let outcome = evaluate_expect_outcome("echo git checkout HEAD -- is dangerous");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn echo_checkout_discard() {
        let outcome = evaluate_expect_outcome("echo git checkout -- is dangerous");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_checkout_head() {
        let outcome = evaluate_expect_outcome("rg 'git checkout HEAD --' README.md");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_checkout_discard() {
        let outcome = evaluate_expect_outcome("rg 'git checkout --' README.md");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
