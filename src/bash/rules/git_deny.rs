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
        let result = eval_rules(git_deny_rules(), "git reset --hard");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_head() {
        let result = eval_rules(git_deny_rules(), "git reset --hard HEAD");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_head_1() {
        let result = eval_rules(git_deny_rules(), "git reset --hard HEAD~1");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_origin_main() {
        let result = eval_rules(git_deny_rules(), "git reset --hard origin/main");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_chained() {
        let result = eval_rules(
            git_deny_rules(),
            "git fetch && git reset --hard origin/main",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset_hard_in_chain() {
        let result = eval_rules(
            git_deny_rules(),
            "git stash && git reset --hard && git stash pop",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_reset() {
        let result = eval_rules(git_deny_rules(), "git reset");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_head() {
        let result = eval_rules(git_deny_rules(), "git reset HEAD");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_soft() {
        let result = eval_rules(git_deny_rules(), "git reset --soft HEAD~1");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_mixed() {
        let result = eval_rules(git_deny_rules(), "git reset --mixed HEAD~1");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_reset_file() {
        let result = eval_rules(git_deny_rules(), "git reset HEAD -- file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_status() {
        let result = eval_rules(git_deny_rules(), "git status");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_reset_hard() {
        let result = eval_rules(git_deny_rules(), "echo git reset --hard is dangerous");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_reset_hard() {
        let result = eval_rules(git_deny_rules(), "rg 'git reset --hard' README.md");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_pop() {
        let result = eval_rules(git_deny_rules(), "git stash pop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop_ref() {
        let result = eval_rules(git_deny_rules(), "git stash pop stash@{0}");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop_index() {
        let result = eval_rules(git_deny_rules(), "git stash pop --index");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_pop_chained() {
        let result = eval_rules(git_deny_rules(), "git stash && git pull && git stash pop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop() {
        let result = eval_rules(git_deny_rules(), "git stash drop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop_ref() {
        let result = eval_rules(git_deny_rules(), "git stash drop stash@{0}");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop_stash_2() {
        let result = eval_rules(git_deny_rules(), "git stash drop stash@{2}");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_drop_chained() {
        let result = eval_rules(git_deny_rules(), "git stash list && git stash drop");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_clear() {
        let result = eval_rules(git_deny_rules(), "git stash clear");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash_clear_chained() {
        let result = eval_rules(git_deny_rules(), "false || git stash clear");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_stash() {
        let result = eval_rules(git_deny_rules(), "git stash");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_push() {
        let result = eval_rules(git_deny_rules(), "git stash push");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_push_m() {
        let result = eval_rules(git_deny_rules(), "git stash push -m 'wip'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_apply() {
        let result = eval_rules(git_deny_rules(), "git stash apply");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_apply_ref() {
        let result = eval_rules(git_deny_rules(), "git stash apply stash@{0}");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_list() {
        let result = eval_rules(git_deny_rules(), "git stash list");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_show() {
        let result = eval_rules(git_deny_rules(), "git stash show");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_show_p() {
        let result = eval_rules(git_deny_rules(), "git stash show -p");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_stash_branch() {
        let result = eval_rules(git_deny_rules(), "git stash branch newbranch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_stash_pop() {
        let result = eval_rules(git_deny_rules(), "echo git stash pop is blocked");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_stash_drop() {
        let result = eval_rules(git_deny_rules(), "rg 'git stash drop' file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cat_stash_clear() {
        let result = eval_rules(git_deny_rules(), "cat stash-clear-notes.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_d_fd() {
        let result = eval_rules(git_deny_rules(), "git clean -fd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_fxd() {
        let result = eval_rules(git_deny_rules(), "git clean -fxd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d() {
        let result = eval_rules(git_deny_rules(), "git clean -d");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_df() {
        let result = eval_rules(git_deny_rules(), "git clean -df");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_dxf() {
        let result = eval_rules(git_deny_rules(), "git clean -dxf");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_d_chained() {
        let result = eval_rules(git_deny_rules(), "ls && git clean -fd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_clean_f() {
        let result = eval_rules(git_deny_rules(), "git clean -f file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_fx() {
        let result = eval_rules(git_deny_rules(), "git clean -fx file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_fx_dash_filename() {
        let result = eval_rules(
            git_deny_rules(),
            "git clean -fx /path/to/some-dash-delimited-file.sh",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_f_dash_path() {
        let result = eval_rules(git_deny_rules(), "git clean -f /path/dir-name/file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_clean_n() {
        let result = eval_rules(git_deny_rules(), "git clean -n");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_git_clean() {
        let result = eval_rules(git_deny_rules(), "echo git clean -fxd");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_discard_head_file() {
        let result = eval_rules(git_deny_rules(), "git checkout HEAD -- file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_dot() {
        let result = eval_rules(git_deny_rules(), "git checkout HEAD -- .");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_src() {
        let result = eval_rules(git_deny_rules(), "git checkout HEAD -- src/");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_multiple() {
        let result = eval_rules(git_deny_rules(), "git checkout HEAD -- file1.txt file2.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_chained_head() {
        let result = eval_rules(
            git_deny_rules(),
            "git status && git checkout HEAD -- file.txt",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_head_in_chain() {
        let result = eval_rules(
            git_deny_rules(),
            "git stash && git checkout HEAD -- . && git stash pop",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_file() {
        let result = eval_rules(git_deny_rules(), "git checkout -- file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_dot() {
        let result = eval_rules(git_deny_rules(), "git checkout -- .");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_src() {
        let result = eval_rules(git_deny_rules(), "git checkout -- src/");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_discard_chained() {
        let result = eval_rules(git_deny_rules(), "git status && git checkout -- file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn git_checkout_branch() {
        let result = eval_rules(git_deny_rules(), "git checkout main");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_b() {
        let result = eval_rules(git_deny_rules(), "git checkout -b new-branch");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_head_1() {
        let result = eval_rules(git_deny_rules(), "git checkout HEAD~1");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_checkout_head_caret() {
        let result = eval_rules(git_deny_rules(), "git checkout HEAD^");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_checkout_head() {
        let result = eval_rules(git_deny_rules(), "echo git checkout HEAD -- is dangerous");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_checkout_discard() {
        let result = eval_rules(git_deny_rules(), "echo git checkout -- is dangerous");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_checkout_head() {
        let result = eval_rules(git_deny_rules(), "rg 'git checkout HEAD --' README.md");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_checkout_discard() {
        let result = eval_rules(git_deny_rules(), "rg 'git checkout --' README.md");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
