//! Rules for `rm`: deny by default, allow removal inside trusted paths.

use crate::prelude::*;
use std::fs::canonicalize;

/// Directories whose existing contents `rm` may remove.
const RM_TRUSTED_PATHS: &[&str] = &["/var/tmp/claude", "/var/tmp/scratch"];

/// Exact flag args permitted when removing inside [`RM_TRUSTED_PATHS`].
const RM_TRUSTED_FLAGS: &[&str] = &["-r", "-f", "-rf", "-fr"];

/// Rules for `rm`.
#[must_use]
pub fn rm_rules() -> Vec<BashRule> {
    vec![rm(), rm__snap_new(), rm__trusted_path()]
}

/// Deny `rm` outside trusted paths, directing to `git rm` or `git clean` instead.
fn rm() -> BashRule {
    BashRule {
        id: "rm".to_owned(),
        command: "rm".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("**/*.snap.new"),
            ArgMatcher::new("**/*.snap.new.*"),
            ArgMatcher::new("**/.*.pending-snap"),
        ]),
        condition: Some(|ctx| !is_rm_path_trusted(ctx)),
        outcome: Outcome::deny(
            "`rm` is blocked. Alternatives: `git rm -f <file>`, `git rm -fx <file>` (gitignored), \
             `git clean -f <file>`, `git clean -fx <file>` (gitignored)",
        ),
        ..Default::default()
    }
}

/// Deny `rm` of pending insta snapshots, directing to `cargo insta` workflow.
fn rm__snap_new() -> BashRule {
    BashRule {
        id: "rm__snap_new".to_owned(),
        command: "rm".to_owned(),
        with_any: Some(vec![
            ArgMatcher::new("**/*.snap.new"),
            ArgMatcher::new("**/*.snap.new.*"),
            ArgMatcher::new("**/.*.pending-snap"),
        ]),
        outcome: Outcome::deny(
            "`rm` of pending insta snapshots is blocked. Use `cargo insta accept` to accept or \
             `cargo insta reject` to reject pending snapshots",
        ),
        ..Default::default()
    }
}

/// Allow `rm` of a single existing file or directory inside [`RM_TRUSTED_PATHS`].
fn rm__trusted_path() -> BashRule {
    BashRule {
        id: "rm__trusted_path".to_owned(),
        command: "rm".to_owned(),
        condition: Some(is_rm_path_trusted),
        outcome: Outcome::allow("Remove an existing file or directory inside a trusted path"),
        ..Default::default()
    }
}

/// Does `rm` remove a single existing path inside [`RM_TRUSTED_PATHS`]?
///
/// - Every flag must exactly match one of [`RM_TRUSTED_FLAGS`]
/// - Exactly one path operand
fn is_rm_path_trusted(ctx: &BashRuleContext) -> bool {
    let (flags, paths): (Vec<&String>, Vec<&String>) =
        ctx.simple.args.iter().partition(|arg| arg.starts_with('-'));
    if !flags
        .iter()
        .all(|flag| RM_TRUSTED_FLAGS.contains(&flag.as_str()))
    {
        return false;
    }
    let [path] = paths.as_slice() else {
        return false;
    };
    RM_TRUSTED_PATHS
        .iter()
        .any(|trusted| is_inside_path(path, trusted))
}

/// Is `path` an existing file or directory strictly inside `trusted`?
///
/// - Rejects characters the shell could expand or unquote, such as `'`, `$`, `*`
/// - Rejects empty, `.` and `..` components, so `//` and a trailing `/` fail
/// - Rejects symlinks, including symlinked parent directories
fn is_inside_path(path: &str, trusted: &str) -> bool {
    if !path.chars().all(is_literal_path_char) {
        return false;
    }
    let Some(relative) = path
        .strip_prefix(trusted)
        .and_then(|rest| rest.strip_prefix('/'))
    else {
        return false;
    };
    if relative
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return false;
    }
    let canonical = match canonicalize(path) {
        Ok(canonical) => canonical,
        Err(error) => {
            trace!(path, %error, "Failed to canonicalize rm path");
            return false;
        }
    };
    canonical == Path::new(path) && (canonical.is_file() || canonical.is_dir())
}

/// Is `c` safe to appear unquoted in a path without shell expansion?
fn is_literal_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::fs::{create_dir, create_dir_all, write};
    use std::os::unix::fs::symlink;
    use tempfile::{Builder, TempDir};

    const SCRATCH: &str = "/var/tmp/scratch";

    #[test]
    fn rm_r() {
        let result = eval_rules(rm_rules(), "rm -r /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_cap_r() {
        let result = eval_rules(rm_rules(), "rm -R /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf() {
        let result = eval_rules(rm_rules(), "rm -rf /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_cap_rf() {
        let result = eval_rules(rm_rules(), "rm -Rf /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_fr() {
        let result = eval_rules(rm_rules(), "rm -fr /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f_cap_r() {
        let result = eval_rules(rm_rules(), "rm -fR /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_recursive() {
        let result = eval_rules(rm_rules(), "rm --recursive /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rfi() {
        let result = eval_rules(rm_rules(), "rm -rfi /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_ir() {
        let result = eval_rules(rm_rules(), "rm -ir /path/to/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_single_file() {
        let result = eval_rules(rm_rules(), "rm file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_multiple_files() {
        let result = eval_rules(rm_rules(), "rm file1.txt file2.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f() {
        let result = eval_rules(rm_rules(), "rm -f file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_i() {
        let result = eval_rules(rm_rules(), "rm -i file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_with_path() {
        let result = eval_rules(rm_rules(), "rm /path/to/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_wildcard() {
        let result = eval_rules(rm_rules(), "rm *.tmp");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_r_chained() {
        let result = eval_rules(rm_rules(), "ls && rm -r /path");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf_or_chain() {
        let result = eval_rules(rm_rules(), "false || rm -rf /tmp/nothing");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_r_semicolon() {
        let result = eval_rules(rm_rules(), "echo hi ; rm -r /path");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_chained_file() {
        let result = eval_rules(rm_rules(), "ls && rm file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_for_do() {
        let result = eval_rules(rm_rules(), "for f in *.tmp; do rm $f; done");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_if_then() {
        let result = eval_rules(rm_rules(), "if true; then rm file.txt; fi");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn rm_if_else() {
        let result = eval_rules(rm_rules(), "if false; then echo hi; else rm file.txt; fi");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn rm_rf_while_do() {
        let result = eval_rules(rm_rules(), "while true; do rm -rf /tmp/nothing; done");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }
    #[test]
    fn rm_tmp_file() {
        let result = eval_rules(rm_rules(), "rm /tmp/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_f_tmp() {
        let result = eval_rules(rm_rules(), "rm -f /tmp/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_rf_tmp() {
        let result = eval_rules(rm_rules(), "rm -rf /tmp/dir");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_multiple() {
        let result = eval_rules(rm_rules(), "rm /tmp/file1 /tmp/file2");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_path_traversal() {
        let result = eval_rules(rm_rules(), "rm /tmp/../etc/passwd");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rm_tmp_mixed() {
        let result = eval_rules(rm_rules(), "rm /tmp/file.txt /home/user/file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn ls() {
        let result = eval_rules(rm_rules(), "ls -la");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_rm() {
        let result = eval_rules(rm_rules(), "git rm file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn git_rm_r() {
        let result = eval_rules(rm_rules(), "git rm -r dir/");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_rm() {
        let result = eval_rules(rm_rules(), "echo rm is blocked");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rg_rm() {
        let result = eval_rules(rm_rules(), "rg rm .");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cat() {
        let result = eval_rules(rm_rules(), "cat file.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn mv() {
        let result = eval_rules(rm_rules(), "mv old.txt new.txt");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_rm() {
        let result = eval_rules(rm_rules(), "cargo rm some-dep");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rm_snap_new() {
        let result = eval_rules(rm_rules(), "rm path/to/foo.snap.new");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_dot_suffix() {
        let result = eval_rules(rm_rules(), "rm path/to/foo.snap.new.42");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_glob() {
        let result = eval_rules(rm_rules(), "rm crates/core/snapshots/foo__bar.snap.new");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_new_mixed_with_other() {
        let result = eval_rules(rm_rules(), "rm foo.snap.new other.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
        assert!(!outcome.reason.contains("git clean"));
    }

    #[test]
    fn rm_pending_snap_inline() {
        let result = eval_rules(rm_rules(), "rm src/.foo.rs.pending-snap");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("cargo insta"));
    }

    #[test]
    fn rm_snap_not_new() {
        let result = eval_rules(rm_rules(), "rm foo.snap");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
        assert!(outcome.reason.contains("git rm"));
    }

    #[test]
    fn xargs_rm() {
        let result = eval_rules(rm_rules(), "echo file | xargs rm");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn rm_trusted_dir_rf() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_dir_fr() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -fr {path}")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_dir_r_f() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -r -f {path}")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_dir_r() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -r {path}")), Decision::Allow);
    }

    /// GNU `rm` accepts flags after operands.
    #[test]
    fn rm_trusted_dir_flags_after_path() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm {path} -rf")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_file() {
        let dir = scratch_dir();
        let path = make_file(&dir, "file.txt");
        assert_eq!(decide(&format!("rm {path}")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_file_f() {
        let dir = scratch_dir();
        let path = make_file(&dir, "file.txt");
        assert_eq!(decide(&format!("rm -f {path}")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_nested() {
        let dir = scratch_dir();
        make_dir(&dir, "a");
        make_dir(&dir, "a/b");
        let path = make_file(&dir, "a/b/file.txt");
        assert_eq!(decide(&format!("rm {path}")), Decision::Allow);
    }

    #[test]
    fn rm_trusted_chained() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        let output = eval_rules(rm_rules(), &format!("rm -rf {path} && rm -rf {path}"));
        assert_eq!(expect_outcome(output).decision, Decision::Allow);
    }

    #[test]
    fn rm_trusted_missing() {
        let dir = scratch_dir();
        let path = format!("{}/missing", dir.path().display());
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_symlink_file() {
        let dir = scratch_dir();
        let target = make_file(&dir, "file.txt");
        let path = make_symlink(&dir, &target, "link");
        assert_eq!(decide(&format!("rm {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_symlink_dir() {
        let dir = scratch_dir();
        let target = make_dir(&dir, "real");
        let path = make_symlink(&dir, &target, "link");
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Deny);
    }

    /// The file is real but reached through a symlinked parent directory.
    #[test]
    fn rm_trusted_symlink_parent() {
        let dir = scratch_dir();
        let target = make_dir(&dir, "real");
        make_file(&dir, "real/file.txt");
        let link = make_symlink(&dir, &target, "link");
        assert_eq!(decide(&format!("rm {link}/file.txt")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_root() {
        assert_eq!(decide(&format!("rm -rf {SCRATCH}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_root_trailing_slash() {
        assert_eq!(decide(&format!("rm -rf {SCRATCH}/")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_trailing_slash() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -rf {path}/")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_parent_component() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -rf {path}/../sub")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_current_component() {
        let dir = scratch_dir();
        make_dir(&dir, "sub");
        let path = format!("{}/./sub", dir.path().display());
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_double_slash() {
        let dir = scratch_dir();
        make_dir(&dir, "sub");
        let path = format!("{}//sub", dir.path().display());
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Deny);
    }

    /// A sibling of the trusted path sharing its prefix.
    #[test]
    fn rm_trusted_prefix_sibling() {
        let sibling = Builder::new()
            .prefix("scratch")
            .tempdir_in("/var/tmp")
            .expect("should create sibling dir");
        let path = make_dir(&sibling, "sub");
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_cap_r() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -R {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_recursive_long() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm --recursive {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_rfv() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -rfv {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_i() {
        let dir = scratch_dir();
        let path = make_file(&dir, "file.txt");
        assert_eq!(decide(&format!("rm -i {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_double_dash() {
        let dir = scratch_dir();
        let path = make_file(&dir, "file.txt");
        assert_eq!(decide(&format!("rm -- {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_two_paths() {
        let dir = scratch_dir();
        let first = make_file(&dir, "a.txt");
        let second = make_file(&dir, "b.txt");
        assert_eq!(decide(&format!("rm {first} {second}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_no_path() {
        assert_eq!(decide("rm -rf"), Decision::Deny);
    }

    #[test]
    fn rm_trusted_relative() {
        assert_eq!(decide("rm -rf var/tmp/scratch/sub"), Decision::Deny);
    }

    #[test]
    fn rm_trusted_variable() {
        assert_eq!(decide("rm -rf /var/tmp/scratch/$NAME"), Decision::Deny);
    }

    #[test]
    fn rm_trusted_glob() {
        let dir = scratch_dir();
        make_dir(&dir, "sub");
        let path = format!("{}/su*", dir.path().display());
        assert_eq!(decide(&format!("rm -rf {path}")), Decision::Deny);
    }

    #[test]
    fn rm_trusted_quoted() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "sub");
        assert_eq!(decide(&format!("rm -rf '{path}'")), Decision::Deny);
    }

    /// Known limitation: a quoted segment is blocked even when the path is safe.
    #[test]
    fn rm_trusted_quoted_segment_with_space() {
        let dir = scratch_dir();
        make_dir(&dir, "spaced path");
        let parent = dir.path().display();
        assert_eq!(
            decide(&format!(r#"rm -rf {parent}/"spaced path""#)),
            Decision::Deny
        );
    }

    /// Known limitation: a fully quoted path is blocked even when the path is safe.
    #[test]
    fn rm_trusted_quoted_path_with_space() {
        let dir = scratch_dir();
        let path = make_dir(&dir, "spaced path");
        assert_eq!(decide(&format!(r#"rm -rf "{path}""#)), Decision::Deny);
    }

    /// Evaluate `command` against [`rm_rules`] and return the decision.
    fn decide(command: &str) -> Decision {
        expect_outcome(eval_rules(rm_rules(), command)).decision
    }

    /// Create a temp dir inside the scratch trusted path.
    fn scratch_dir() -> TempDir {
        create_dir_all(SCRATCH).expect("should create scratch dir");
        Builder::new()
            .tempdir_in(SCRATCH)
            .expect("should create temp dir")
    }

    /// Create a directory at `name` inside `dir` and return its path.
    fn make_dir(dir: &TempDir, name: &str) -> String {
        let path = dir.path().join(name);
        create_dir(&path).expect("should create dir");
        path.display().to_string()
    }

    /// Create a file at `name` inside `dir` and return its path.
    fn make_file(dir: &TempDir, name: &str) -> String {
        let path = dir.path().join(name);
        write(&path, "content").expect("should write file");
        path.display().to_string()
    }

    /// Create a symlink at `name` inside `dir` pointing to `target` and return its path.
    fn make_symlink(dir: &TempDir, target: &str, name: &str) -> String {
        let path = dir.path().join(name);
        symlink(target, &path).expect("should create symlink");
        path.display().to_string()
    }
}
