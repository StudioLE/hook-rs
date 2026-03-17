//! Glob-based rule for matching file and search paths.

use crate::prelude::*;

/// Rule that matches paths against an exact string and/or a compiled glob pattern.
///
/// - Literal paths match via exact string comparison
/// - Directory prefixes match for patterns ending in `/**` or `/**/*`
/// - Patterns without `/` match against the filename component only
#[derive(Debug)]
pub struct PathRule {
    /// Exact string for literal or directory-prefix matching.
    exact: Option<String>,
    /// Compiled glob pattern for file-level matching.
    matcher: Option<GlobMatcher>,
    /// Match against only the filename component of the path.
    is_filename: bool,
}

impl PathRule {
    /// Create a new [`PathRule`] from optional exact and glob components.
    pub(crate) fn new(
        exact: Option<String>,
        matcher: Option<GlobMatcher>,
        is_filename: bool,
    ) -> Self {
        Self {
            exact,
            matcher,
            is_filename,
        }
    }

    /// Test whether the given path matches this rule's exact string or glob.
    pub fn is_match(&self, path: &str) -> bool {
        let target = if self.is_filename {
            let Some(name) = Path::new(path).file_name() else {
                return false;
            };
            &name.to_string_lossy()
        } else {
            path
        };
        self.is_exact_match(target) || self.is_glob_match(target)
    }

    /// Test whether the given path matches this rule's exact string.
    fn is_exact_match(&self, path: &str) -> bool {
        let is_match = self.exact.as_ref().is_some_and(|exact| exact == path);
        if is_match {
            trace!(path = %path, "Exact match");
        }
        is_match
    }

    /// Test whether the given path matches this rule's glob pattern.
    fn is_glob_match(&self, path: &str) -> bool {
        let is_match = self
            .matcher
            .as_ref()
            .is_some_and(|matcher| matcher.is_match(path));
        if is_match {
            trace!(path = %path, glob = %self.matcher.as_ref().expect("glob is set").glob(), "Glob match");
        }
        is_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(pattern: &str) -> PathRule {
        let home = PathBuf::from("/home/user");
        let factory = PathRuleFactory::new(home);
        factory.create(pattern)
    }

    #[test]
    fn double_star_matches_directory() {
        let r = rule("/opt/data/**");
        assert!(r.is_match("/opt/data"));
    }

    #[test]
    fn double_star_star_matches_directory() {
        let r = rule("/src/**/*");
        assert!(r.is_match("/src"));
    }

    #[test]
    fn unrelated_directory_no_match() {
        let r = rule("/opt/data/**");
        assert!(!r.is_match("/etc"));
    }

    #[test]
    fn matches_file_via_glob() {
        let r = rule("/opt/data/**");
        assert!(r.is_match("/opt/data/file.txt"));
    }

    #[test]
    fn matches_directory_via_prefix() {
        let r = rule("/opt/data/**");
        assert!(r.is_match("/opt/data"));
    }

    #[test]
    fn no_match_unrelated() {
        let r = rule("/opt/data/**");
        assert!(!r.is_match("/etc/passwd"));
    }

    // ** recursive matching

    #[test]
    fn double_star_matches_nested() {
        let r = rule("/opt/data/**");
        assert!(r.is_match("/opt/data/a/b/c/file.txt"));
    }

    #[test]
    fn double_star_matches_direct_child() {
        let r = rule("/opt/data/**");
        assert!(r.is_match("/opt/data/file.txt"));
    }

    #[test]
    fn double_star_no_match_sibling() {
        let r = rule("/opt/data/**");
        assert!(!r.is_match("/opt/other/file.txt"));
    }

    // * single-level matching (literal_separator makes * stop at /)

    #[test]
    fn single_star_matches_one_level() {
        let r = rule("/opt/*/file.txt");
        assert!(r.is_match("/opt/data/file.txt"));
    }

    #[test]
    fn single_star_no_match_nested() {
        let r = rule("/opt/*/file.txt");
        assert!(!r.is_match("/opt/a/b/file.txt"));
    }

    // *.ext extension matching

    #[test]
    fn star_ext_matches_in_dir() {
        let r = rule("/tmp/*.rs");
        assert!(r.is_match("/tmp/lib.rs"));
    }

    #[test]
    fn star_ext_no_match_subdirectory() {
        let r = rule("/tmp/*.rs");
        assert!(!r.is_match("/tmp/src/lib.rs"));
    }

    #[test]
    fn star_ext_no_match_wrong_extension() {
        let r = rule("/tmp/*.rs");
        assert!(!r.is_match("/tmp/lib.toml"));
    }

    #[test]
    fn double_star_ext_matches_nested() {
        let r = rule("/src/**/*.rs");
        assert!(r.is_match("/src/rules/read.rs"));
    }

    #[test]
    fn double_star_ext_matches_deep_nested() {
        let r = rule("/src/**/*.rs");
        assert!(r.is_match("/src/a/b/c/lib.rs"));
    }

    #[test]
    fn double_star_ext_no_match_wrong_extension() {
        let r = rule("/src/**/*.rs");
        assert!(!r.is_match("/src/rules/read.toml"));
    }

    // exact path

    #[test]
    fn exact_path_matches() {
        let r = rule("/etc/hosts");
        assert!(r.is_match("/etc/hosts"));
    }

    #[test]
    fn exact_path_no_match_different() {
        let r = rule("/etc/hosts");
        assert!(!r.is_match("/etc/passwd"));
    }

    #[test]
    fn exact_path_no_match_nested() {
        let r = rule("/etc/hosts");
        assert!(!r.is_match("/etc/hosts/extra"));
    }

    // basename matching (patterns without /)

    #[test]
    fn bare_filename_matches_anywhere() {
        let r = rule("CLAUDE.md");
        assert!(r.is_match("/home/user/project/.claude/CLAUDE.md"));
        assert!(r.is_match("/tmp/CLAUDE.md"));
    }

    #[test]
    fn bare_filename_no_match_different_name() {
        let r = rule("CLAUDE.md");
        assert!(!r.is_match("/home/user/README.md"));
    }

    #[test]
    fn bare_glob_matches_basename() {
        let r = rule("*.md");
        assert!(r.is_match("/home/user/project/README.md"));
        assert!(r.is_match("/tmp/CLAUDE.md"));
    }

    #[test]
    fn bare_glob_no_match_wrong_extension() {
        let r = rule("*.md");
        assert!(!r.is_match("/home/user/project/lib.rs"));
    }

    #[test]
    fn bare_dotfile_pattern() {
        let r = rule(".env");
        assert!(r.is_match("/home/user/project/.env"));
    }

    #[test]
    fn bare_dotfile_glob() {
        let r = rule(".env.*");
        assert!(r.is_match("/home/user/project/.env.local"));
        assert!(r.is_match("/tmp/.env.production"));
    }

    #[test]
    fn bare_dotfile_glob_no_match_bare_env() {
        let r = rule(".env.*");
        assert!(!r.is_match("/home/user/project/.env"));
    }
}
