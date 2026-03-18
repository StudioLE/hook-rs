//! Rules for `find` operations: allow read-only, deny destructive.

use crate::prelude::*;

/// Deny destructive `find`, allow read-only `find`.
pub fn find_rules() -> Vec<BashRule> {
    vec![find_delete(), find_exec_rm(), find__read_only()]
}

/// Allow `find` without destructive flags when the path is in the read allow list.
fn find__read_only() -> BashRule {
    BashRule {
        id: "find__read_only".to_owned(),
        command: "find".to_owned(),
        without_any: Some(vec![
            Arg::new("-delete"),
            Arg::new("-exec"),
            Arg::new("-execdir"),
        ]),
        condition: Some(find_path_allowed),
        outcome: Outcome::allow("Safe command: find (read-only, allowed path)"),
        ..Default::default()
    }
}

/// Check that `find`'s path argument is in the read allow list.
fn find_path_allowed(
    context: &SimpleContext,
    _complete: &CompleteContext,
    settings: &Settings,
) -> bool {
    let path = context.args.first().filter(|a| !a.starts_with('-'));
    let Some(path) = path else {
        return false;
    };
    let path = unquote_str(path);
    let factory = PathRuleFactory::default();
    if let Some(is_allowed) = factory.is_match(&path, &settings.read.paths) {
        trace!(is_allowed, "find path matched read allow list");
        is_allowed
    } else {
        trace!("find path not in read allow list");
        false
    }
}

/// Deny `find -delete`.
fn find_delete() -> BashRule {
    BashRule {
        id: "find_delete".to_owned(),
        command: "find".to_owned(),
        with_any: Some(vec![Arg::new("-delete")]),
        outcome: Outcome::deny(
            "find -delete is blocked. Use 'find ... -print' to preview matches first, \
             then delete with targeted commands.",
        ),
        ..Default::default()
    }
}

/// Deny `find -exec rm`.
fn find_exec_rm() -> BashRule {
    BashRule {
        id: "find_exec_rm".to_owned(),
        command: "find".to_owned(),
        with_any: Some(vec![
            Arg::new("-exec").value("rm"),
            Arg::new("-execdir").value("rm"),
        ]),
        outcome: Outcome::deny(
            "find -exec rm is blocked. Use 'find ... -print' to preview matches first, \
             then delete with targeted commands.",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn _find_delete() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_delete__path() {
        let outcome = evaluate_expect_outcome("find /path -type f -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_delete__redirect() {
        let outcome = evaluate_expect_outcome("find . -name .lock -delete 2>/dev/null");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_exec_rm() {
        let outcome = evaluate_expect_outcome("find . -name '*.tmp' -exec rm {} \\;");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_exec_rm__f() {
        let outcome = evaluate_expect_outcome("find . -type f -exec rm -f {} +");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_exec_rm__execdir() {
        let outcome = evaluate_expect_outcome("find . -name '*.log' -execdir rm {} \\;");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_delete__chained() {
        let outcome = evaluate_expect_outcome("ls && find . -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_delete__semicolon() {
        let outcome = evaluate_expect_outcome("echo test ; find . -name '*.tmp' -delete");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn _find_name() {
        let reason = evaluate_expect_skip("find . -name '*.rs'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_print() {
        let reason = evaluate_expect_skip("find . -type f -print");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_maxdepth() {
        let reason = evaluate_expect_skip("find /path -maxdepth 1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_exec_ls() {
        let reason = evaluate_expect_skip("find . -name '*.tmp' -exec ls {} \\;");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_exec_cat() {
        let reason = evaluate_expect_skip("find . -name '*.txt' -exec cat {} +");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_no_path() {
        let reason = evaluate_expect_skip("find -name '*.rs'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _find_trusted_path() {
        let home = dirs::home_dir().expect("home directory should be resolvable");
        let path = home.join(".cargo/registry/src/crates");
        let cmd = format!("find {} -type f -name '*.rs'", path.display());
        let outcome = evaluate_expect_outcome(&cmd);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _find_trusted_path__piped() {
        let home = dirs::home_dir().expect("home directory should be resolvable");
        let path = home.join(".rustup/toolchains/stable");
        let cmd = format!("find {} -type f -name '*.md' | head -20", path.display());
        let outcome = evaluate_expect_outcome(&cmd);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _find_trusted_path__exec_ls() {
        let home = dirs::home_dir().expect("home directory should be resolvable");
        let path = home.join(".cargo/registry/src/crates");
        let cmd = format!("find {} -name '*.toml' -exec ls {{}} \\;", path.display());
        let reason = evaluate_expect_skip(&cmd);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn _echo_find_delete() {
        let outcome = evaluate_expect_outcome("echo 'find -delete is dangerous'");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _grep_delete() {
        let outcome = evaluate_expect_outcome("grep -r 'delete' .");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn _git_log_grep_delete() {
        // git log is Allow via git_approval, grep is Allow via safe_rules
        let outcome = evaluate_expect_outcome("git log --oneline | grep delete");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
