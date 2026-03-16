//! Glob-based rule for matching grep search paths, including directory prefixes.

use crate::prelude::*;
use globset::GlobMatcher;

/// Rule that matches search paths against a compiled glob pattern.
///
/// Supports both exact file matching via the glob and directory prefix matching
/// for patterns ending in `/**` or `/**/*`.
#[derive(Debug)]
pub struct GrepRule {
    /// Compiled glob pattern for file-level matching.
    matcher: GlobMatcher,
    /// Raw expanded pattern string (after tilde expansion).
    pattern: String,
    /// Permission decision for a matched path.
    pub outcome: Outcome,
}

impl FromGlob for GrepRule {
    fn from_glob(matcher: GlobMatcher, pattern: String, outcome: Outcome) -> Self {
        Self {
            matcher,
            pattern,
            outcome,
        }
    }
}

impl GrepRule {
    /// Test whether the given path matches this rule's glob or directory prefix.
    pub fn matches(&self, path: &str) -> bool {
        self.matches_directory(path) || self.matcher.is_match(path)
    }

    /// Test whether a directory path is covered by a recursive glob pattern.
    ///
    /// Strips `/**` or `/**/*` suffix from the pattern and checks if the path
    /// starts with the resulting base prefix. Returns `false` for patterns
    /// without a recursive glob suffix.
    fn matches_directory(&self, dir_path: &str) -> bool {
        let base = self
            .pattern
            .strip_suffix("/**/*")
            .or_else(|| self.pattern.strip_suffix("/**"));
        match base {
            Some(prefix) => dir_path.starts_with(prefix),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use globset::GlobBuilder;

    fn rule(pattern: &str) -> GrepRule {
        let matcher = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .expect("pattern should be valid")
            .compile_matcher();
        GrepRule::from_glob(matcher, pattern.to_owned(), Outcome::allow("test"))
    }

    // matches_directory

    #[test]
    fn double_star_matches_directory() {
        let r = rule("/opt/data/**");
        assert!(r.matches_directory("/opt/data"));
    }

    #[test]
    fn double_star_star_matches_directory() {
        let r = rule("/src/**/*");
        assert!(r.matches_directory("/src"));
    }

    #[test]
    fn exact_path_no_directory_match() {
        let r = rule("/etc/hosts");
        assert!(!r.matches_directory("/etc/hosts"));
    }

    #[test]
    fn unrelated_directory_no_match() {
        let r = rule("/opt/data/**");
        assert!(!r.matches_directory("/etc"));
    }

    // matches (combined)

    #[test]
    fn matches_file_via_glob() {
        let r = rule("/opt/data/**");
        assert!(r.matches("/opt/data/file.txt"));
    }

    #[test]
    fn matches_directory_via_prefix() {
        let r = rule("/opt/data/**");
        assert!(r.matches("/opt/data"));
    }

    #[test]
    fn no_match_unrelated() {
        let r = rule("/opt/data/**");
        assert!(!r.matches("/etc/passwd"));
    }
}
