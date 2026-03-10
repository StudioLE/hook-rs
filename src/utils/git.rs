//! Git-specific utilities for command analysis.

use crate::command::{self, SimpleContext};

/// Parsed git command arguments with optional `-C <path>` extracted.
#[derive(Debug)]
pub struct GitArgs<'a> {
    /// Repository path from `-C <path>`, if present.
    pub path: Option<String>,
    /// Remaining arguments after stripping `-C <path>`.
    pub args: &'a [String],
}

/// Extract git subcommand arguments, stripping `-C <path>` if present.
///
/// - Returns `None` for non-git commands
#[must_use]
pub fn parse_git_args(cmd: &SimpleContext) -> Option<GitArgs<'_>> {
    if cmd.name != "git" {
        return None;
    }
    let args = &cmd.args;
    if args.first().is_some_and(|a| a == "-C") {
        let path = args
            .get(1)
            .map(|a| command::unquote(a).trim_end_matches('/').to_owned());
        Some(GitArgs {
            path,
            args: args.get(2..).unwrap_or_default(),
        })
    } else {
        Some(GitArgs { path: None, args })
    }
}

/// Classification of a git repository path by trust level.
#[derive(Debug, PartialEq, Eq)]
pub enum PathClass {
    /// No path specified (local repo).
    None,
    /// Path under the trusted repository root.
    Trusted,
    /// Path under the forked repository directory.
    Forked,
    /// Unknown or untrusted path.
    Unknown,
}

/// Classify a repository path by trust level.
#[must_use]
pub fn classify_path(path: &str) -> PathClass {
    if path.is_empty() {
        PathClass::None
    } else if path.starts_with("/var/mnt/e/Repos/Forked/") || path == "/var/mnt/e/Repos/Forked" {
        PathClass::Forked
    } else if path.starts_with("/var/mnt/e/Repos/") || path == "/var/mnt/e/Repos" {
        PathClass::Trusted
    } else {
        PathClass::Unknown
    }
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "test assertions — panic is a test failure"
)]
mod tests {
    use super::*;
    use crate::command::CompleteContext;

    #[test]
    fn git_c_path() {
        let p = CompleteContext::parse("git -C /var/mnt/e/Repos/Rust/caesura status")
            .expect("should parse");
        let cmd = &p.children[0].children[0];
        let ga = parse_git_args(cmd).expect("should parse");
        assert_eq!(ga.path.as_deref(), Some("/var/mnt/e/Repos/Rust/caesura"));
        assert_eq!(ga.args[0], "status");
    }

    #[test]
    fn git_c_quoted_path() {
        let p = CompleteContext::parse("git -C \"/var/mnt/e/Repos/Rust/caesura\" status")
            .expect("should parse");
        let cmd = &p.children[0].children[0];
        let ga = parse_git_args(cmd).expect("should parse");
        assert_eq!(ga.path.as_deref(), Some("/var/mnt/e/Repos/Rust/caesura"));
        assert_eq!(ga.args[0], "status");
    }

    #[test]
    fn git_c_single_quoted() {
        let p = CompleteContext::parse("git -C '/var/mnt/e/Repos/Rust/caesura' status")
            .expect("should parse");
        let cmd = &p.children[0].children[0];
        let ga = parse_git_args(cmd).expect("should parse");
        assert_eq!(ga.path.as_deref(), Some("/var/mnt/e/Repos/Rust/caesura"));
        assert_eq!(ga.args[0], "status");
    }

    #[test]
    fn git_c_trailing_slash() {
        let p = CompleteContext::parse("git -C /var/mnt/e/Repos/Rust/caesura/ status")
            .expect("should parse");
        let cmd = &p.children[0].children[0];
        let ga = parse_git_args(cmd).expect("should parse");
        assert_eq!(ga.path.as_deref(), Some("/var/mnt/e/Repos/Rust/caesura"));
    }

    #[test]
    fn git_no_path() {
        let p = CompleteContext::parse("git status").expect("should parse");
        let cmd = &p.children[0].children[0];
        let ga = parse_git_args(cmd).expect("should parse");
        assert!(ga.path.is_none());
        assert_eq!(ga.args[0], "status");
    }

    #[test]
    fn non_git() {
        let p = CompleteContext::parse("ls -la").expect("should parse");
        let cmd = &p.children[0].children[0];
        assert!(parse_git_args(cmd).is_none());
    }
}
