//! Glob-based rule for matching file paths.

use crate::prelude::*;
use globset::GlobMatcher;

/// Rule that matches file paths against a compiled glob pattern.
#[derive(Debug)]
pub struct ReadRule {
    /// Compiled glob pattern.
    pub matcher: GlobMatcher,
    /// Permission decision for a matched path.
    pub outcome: Outcome,
}

impl ReadRule {
    /// Create a rule from a pre-compiled matcher and outcome.
    pub fn new(matcher: GlobMatcher, outcome: Outcome) -> Self {
        Self { matcher, outcome }
    }

    /// Test whether the given file path matches this rule.
    pub fn matches(&self, file_path: &str) -> bool {
        self.matcher.is_match(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use globset::GlobBuilder;

    fn rule(pattern: &str) -> ReadRule {
        let matcher = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .expect("pattern should be valid")
            .compile_matcher();
        ReadRule::new(matcher, Outcome::allow("test"))
    }

    // ** recursive matching

    #[test]
    fn double_star_matches_nested() {
        let r = rule("/opt/data/**");
        assert!(r.matches("/opt/data/a/b/c/file.txt"));
    }

    #[test]
    fn double_star_matches_direct_child() {
        let r = rule("/opt/data/**");
        assert!(r.matches("/opt/data/file.txt"));
    }

    #[test]
    fn double_star_no_match_sibling() {
        let r = rule("/opt/data/**");
        assert!(!r.matches("/opt/other/file.txt"));
    }

    #[test]
    fn double_star_no_match_parent() {
        let r = rule("/opt/data/**");
        assert!(!r.matches("/opt/data"));
    }

    // * single-level matching (literal_separator makes * stop at /)

    #[test]
    fn single_star_matches_one_level() {
        let r = rule("/opt/*/file.txt");
        assert!(r.matches("/opt/data/file.txt"));
    }

    #[test]
    fn single_star_no_match_nested() {
        let r = rule("/opt/*/file.txt");
        assert!(!r.matches("/opt/a/b/file.txt"));
    }

    // *.ext extension matching

    #[test]
    fn star_ext_matches_in_dir() {
        let r = rule("/tmp/*.rs");
        assert!(r.matches("/tmp/lib.rs"));
    }

    #[test]
    fn star_ext_no_match_subdirectory() {
        let r = rule("/tmp/*.rs");
        assert!(!r.matches("/tmp/src/lib.rs"));
    }

    #[test]
    fn star_ext_no_match_wrong_extension() {
        let r = rule("/tmp/*.rs");
        assert!(!r.matches("/tmp/lib.toml"));
    }

    #[test]
    fn double_star_ext_matches_nested() {
        let r = rule("/src/**/*.rs");
        assert!(r.matches("/src/rules/read.rs"));
    }

    #[test]
    fn double_star_ext_matches_deep_nested() {
        let r = rule("/src/**/*.rs");
        assert!(r.matches("/src/a/b/c/lib.rs"));
    }

    #[test]
    fn double_star_ext_no_match_wrong_extension() {
        let r = rule("/src/**/*.rs");
        assert!(!r.matches("/src/rules/read.toml"));
    }

    // exact path

    #[test]
    fn exact_path_matches() {
        let r = rule("/etc/hosts");
        assert!(r.matches("/etc/hosts"));
    }

    #[test]
    fn exact_path_no_match_different() {
        let r = rule("/etc/hosts");
        assert!(!r.matches("/etc/passwd"));
    }

    #[test]
    fn exact_path_no_match_nested() {
        let r = rule("/etc/hosts");
        assert!(!r.matches("/etc/hosts/extra"));
    }
}
