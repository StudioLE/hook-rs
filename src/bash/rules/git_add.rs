//! Allow rule for `git add` when nothing is staged.

use crate::prelude::*;
use std::process::Command;

/// Pathspecs that stage the whole repository or working directory.
const ROOT_PATHSPECS: &[&str] = &[".", "./", ":/", "*"];

/// Staged statuses left by commands that are already allowed.
///
/// - `R100` is a pure rename from `git mv`
/// - `D` is a deletion from `git rm`
const IGNORED_STATUSES: &[&str] = &["R100", "D"];

/// Allow `git add` of specific paths when nothing is staged.
///
/// - Protects files the user staged after review from being mixed with Claude's
/// - Ignores staged moves and deletions from `git mv` and `git rm`
/// - Rejects `-A`, `-u`, `-f`, interactive flags, and root pathspecs like `.`
pub fn git_add__nothing_staged() -> BashRule {
    BashRule {
        id: "git_add__nothing_staged".to_owned(),
        command: "git add".to_owned(),
        condition: Some(is_safe_add),
        outcome: Outcome::allow("Safe `git add` with nothing staged"),
        ..Default::default()
    }
}

/// Are the args safe and is nothing staged in the working directory?
fn is_safe_add(ctx: &BashRuleContext) -> bool {
    if ctx.simple.has_variable || ctx.simple.contains_substitution {
        trace!("Unknown args");
        return false;
    }
    if !has_safe_args(&ctx.simple.args) {
        trace!("Unsafe args");
        return false;
    }
    let Some(cwd) = &ctx.cwd else {
        trace!("No working directory");
        return false;
    };
    match get_staged_statuses(cwd) {
        Ok(statuses) => statuses
            .iter()
            .all(|status| IGNORED_STATUSES.contains(&status.as_str())),
        Err(report) => {
            warn!("{}", report.render());
            false
        }
    }
}

/// Do the args only use allowed options and exclude root pathspecs?
fn has_safe_args(args: &[String]) -> bool {
    let Ok(parsed) = parse_add_args(args) else {
        return false;
    };
    let Some(add) = parsed.get(1) else {
        return false;
    };
    !add.operands
        .iter()
        .any(|operand| ROOT_PATHSPECS.contains(&operand.as_str()))
}

/// Parse `git add` args using a schema that only allows non-bulk, non-interactive options.
fn parse_add_args(args: &[String]) -> Result<ParsedCommand, Report<CommandParseError>> {
    let schema = CommandSchemaBuilder::new("git")
        .with_subcommand(
            CommandSchemaBuilder::new("add")
                .with_option(OptionSchemaBuilder::new(["-v", "--verbose"]).build())
                .with_option(OptionSchemaBuilder::new(["-n", "--dry-run"]).build())
                .with_option(OptionSchemaBuilder::new(["-N", "--intent-to-add"]).build())
                .with_operand(
                    OperandSchemaBuilder::new("pathspec")
                        .with_variadic()
                        .build(),
                )
                .build(),
        )
        .build();
    CommandParser::new(schema).parse(args.to_vec())
}

/// Get the status of each staged entry in the repository at `cwd`.
///
/// - Returns the first column of `git diff --cached --name-status`, such as `M` or `R100`
/// - Passes `--find-renames` so `diff.renames=false` cannot split a move into `D` and `A`
fn get_staged_statuses(cwd: &str) -> Result<Vec<String>, Report<GitAddError>> {
    let output = Command::new("git")
        .args([
            "--no-optional-locks",
            "diff",
            "--cached",
            "--name-status",
            "--find-renames",
        ])
        .current_dir(cwd)
        .output()
        .change_context(GitAddError::RunGit)
        .attach("cwd", cwd)?;
    if !output.status.success() {
        return Err(Report::new(GitAddError::GitFailed)
            .attach("cwd", cwd)
            .attach("stderr", String::from_utf8_lossy(&output.stderr).trim()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let statuses = stdout
        .lines()
        .map(|line| line.split('\t').next().unwrap_or_default().to_owned())
        .collect();
    Ok(statuses)
}

/// Errors returned by [`get_staged_statuses`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
enum GitAddError {
    /// `git` could not be spawned.
    #[error("run git diff")]
    RunGit,
    /// `git` exited with a non-zero status.
    #[error("git diff failed")]
    GitFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn has_safe_args_paths() {
        for command in [
            "add src/main.rs",
            "add src/main.rs src/lib.rs",
            "add src/",
            "add '*.rs'",
            "add -v src/main.rs",
            "add --verbose src/main.rs",
            "add -n src/main.rs",
            "add --dry-run src/main.rs",
            "add -N src/main.rs",
            "add --intent-to-add src/main.rs",
            "add -vn src/main.rs",
            "add -- src/main.rs",
            "add -- -A",
        ] {
            assert!(has_safe_args(&split(command)), "{command}");
        }
    }

    #[test]
    fn has_safe_args_bulk_options() {
        for command in [
            "add -A",
            "add --all",
            "add --al",
            "add -u",
            "add --update",
            "add --no-ignore-removal",
            "add -vA src/main.rs",
        ] {
            assert!(!has_safe_args(&split(command)), "{command}");
        }
    }

    #[test]
    fn has_safe_args_unsafe_options() {
        for command in [
            "add -f .env",
            "add --force .env",
            "add -fA",
            "add -p",
            "add --patch",
            "add -i",
            "add --interactive",
            "add -e",
            "add --edit",
            "add --pathspec-from-file=list.txt",
        ] {
            assert!(!has_safe_args(&split(command)), "{command}");
        }
    }

    #[test]
    fn has_safe_args_root_pathspecs() {
        for command in [
            "add .",
            "add ./",
            "add :/",
            "add '*'",
            "add \".\"",
            "add src/main.rs .",
            "add -- .",
        ] {
            assert!(!has_safe_args(&split(command)), "{command}");
        }
    }

    /// Git global options before `add` are not in the schema.
    #[test]
    fn has_safe_args_git_options() {
        assert!(!has_safe_args(&split("-c core.editor=vim add src/main.rs")));
    }

    #[test]
    fn git_add_no_cwd() {
        let result = eval_rules(vec![git_add__nothing_staged()], "git add src/main.rs");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn git_add_nothing_staged() {
        // Arrange
        let repo = TestRepo::new();
        repo.write("a.txt", "changed");

        // Act
        let result = repo.eval("git add a.txt");

        // Assert
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn git_add_staged_move_and_delete() {
        // Arrange
        let repo = TestRepo::new();
        repo.git(&["mv", "a.txt", "moved.txt"]);
        repo.git(&["rm", "--quiet", "b.txt"]);
        repo.write("moved.txt", "changed");

        // Act
        let result = repo.eval("git add moved.txt");

        // Assert
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn git_add_staged_modification() {
        // Arrange
        let repo = TestRepo::new();
        repo.write("a.txt", "reviewed");
        repo.git(&["add", "a.txt"]);
        repo.write("b.txt", "changed");

        // Act
        let result = repo.eval("git add b.txt");

        // Assert
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn git_add_staged_new_file() {
        // Arrange
        let repo = TestRepo::new();
        repo.write("new.txt", "reviewed");
        repo.git(&["add", "new.txt"]);

        // Act
        let result = repo.eval("git add a.txt");

        // Assert
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// A staged move with a staged edit is below 100% similarity.
    #[test]
    #[ignore = "creates git repos"]
    fn git_add_staged_move_with_edit() {
        // Arrange
        let repo = TestRepo::new();
        repo.git(&["mv", "a.txt", "moved.txt"]);
        repo.write("moved.txt", "line 1\nline 2\nline 3\nline 4\nchanged\n");
        repo.git(&["add", "moved.txt"]);

        // Act
        let result = repo.eval("git add b.txt");

        // Assert
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn git_add_bulk_option_nothing_staged() {
        let repo = TestRepo::new();
        let result = repo.eval("git add -A");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// The `-C` path is used as the working directory.
    #[test]
    #[ignore = "creates git repos"]
    fn git_add_git_c_trusted() {
        // Arrange
        let repo = TestRepo::new();
        repo.write("a.txt", "changed");
        let parent = repo.dir.path().parent().expect("should have parent");
        let pattern = format!("{}/**", parent.to_string_lossy());
        let settings = Settings::with_git(&[&pattern]);
        let command = format!("git -C {} add a.txt", repo.cwd());

        // Act
        let result = eval_rules_with_settings(git_c_rules(), &command, settings);

        // Assert
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// The variable's value is unknown, so it could expand to `-A` or `.`.
    #[test]
    #[ignore = "creates git repos"]
    fn git_add_variable_nothing_staged() {
        let repo = TestRepo::new();
        let result = repo.eval("git add $file");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn git_add_not_a_repo() {
        // Arrange
        let dir = TempDir::new().expect("should create temp dir");
        let cwd = dir.path().to_string_lossy().to_string();

        // Act
        let result =
            eval_rules_with_cwd(vec![git_add__nothing_staged()], "git add a.txt", Some(cwd));

        // Assert
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn get_staged_statuses_move_delete_modify() {
        // Arrange
        let repo = TestRepo::new();
        repo.git(&["mv", "a.txt", "moved.txt"]);
        repo.git(&["rm", "--quiet", "b.txt"]);
        repo.write("c.txt", "changed");
        repo.git(&["add", "c.txt"]);

        // Act
        let mut statuses = get_staged_statuses(&repo.cwd()).expect("should get statuses");

        // Assert
        statuses.sort();
        assert_eq!(statuses, vec!["D", "M", "R100"]);
    }

    #[test]
    #[ignore = "creates git repos"]
    fn get_staged_statuses_move_with_edit() {
        // Arrange
        let repo = TestRepo::new();
        repo.git(&["mv", "a.txt", "moved.txt"]);
        repo.write("moved.txt", "line 1\nline 2\nline 3\nline 4\nchanged\n");
        repo.git(&["add", "moved.txt"]);

        // Act
        let statuses = get_staged_statuses(&repo.cwd()).expect("should get statuses");

        // Assert
        assert_eq!(statuses, vec!["R077"]);
    }

    fn split(command: &str) -> Vec<String> {
        command.split_whitespace().map(str::to_owned).collect()
    }

    /// Throwaway repository with committed `a.txt`, `b.txt`, and `c.txt`.
    ///
    /// - Ignores global and system git config so user settings cannot interfere
    struct TestRepo {
        dir: TempDir,
    }

    impl TestRepo {
        fn new() -> Self {
            let repo = Self {
                dir: TempDir::new().expect("should create temp dir"),
            };
            repo.git(&["init", "--quiet"]);
            for name in ["a.txt", "b.txt", "c.txt"] {
                repo.write(name, &format!("{name}\nline 1\nline 2\nline 3\nline 4\n"));
            }
            repo.git(&["add", "--all"]);
            repo.git(&["commit", "--quiet", "--message", "initial"]);
            repo
        }

        fn cwd(&self) -> String {
            self.dir.path().to_string_lossy().to_string()
        }

        fn write(&self, name: &str, contents: &str) {
            fs::write(self.dir.path().join(name), contents).expect("should write file");
        }

        fn git(&self, args: &[&str]) {
            let status = Command::new("git")
                .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
                .args(args)
                .current_dir(self.dir.path())
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .status()
                .expect("should run git");
            assert!(status.success(), "git {args:?}");
        }

        fn eval(&self, command: &str) -> Result<Outcome, Report<ParseError>> {
            eval_rules_with_cwd(vec![git_add__nothing_staged()], command, Some(self.cwd()))
        }
    }
}
