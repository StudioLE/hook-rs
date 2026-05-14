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

    #[test]
    fn double_star_directory() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(rule.is_match("/opt/data"));
    }

    #[test]
    fn double_star_star_directory() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/src/**/*");
        assert!(rule.is_match("/src"));
    }

    #[test]
    fn unrelated_directory_etc() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(!rule.is_match("/etc"));
    }

    #[test]
    fn file_via_glob() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(rule.is_match("/opt/data/file.txt"));
    }

    #[test]
    fn directory_via_prefix() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(rule.is_match("/opt/data"));
    }

    #[test]
    fn unrelated_passwd() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(!rule.is_match("/etc/passwd"));
    }

    #[test]
    fn double_star_nested() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(rule.is_match("/opt/data/a/b/c/file.txt"));
    }

    #[test]
    fn double_star_direct_child() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(rule.is_match("/opt/data/file.txt"));
    }

    #[test]
    fn double_star_sibling() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/data/**");
        assert!(!rule.is_match("/opt/other/file.txt"));
    }

    #[test]
    fn single_star_one_level() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/*/file.txt");
        assert!(rule.is_match("/opt/data/file.txt"));
    }

    #[test]
    fn single_star_nested() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/opt/*/file.txt");
        assert!(!rule.is_match("/opt/a/b/file.txt"));
    }

    #[test]
    fn star_ext_in_dir() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/tmp/*.rs");
        assert!(rule.is_match("/tmp/lib.rs"));
    }

    #[test]
    fn star_ext_subdirectory() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/tmp/*.rs");
        assert!(!rule.is_match("/tmp/src/lib.rs"));
    }

    #[test]
    fn star_ext_wrong_extension() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/tmp/*.rs");
        assert!(!rule.is_match("/tmp/lib.toml"));
    }

    #[test]
    fn double_star_ext_nested() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/src/**/*.rs");
        assert!(rule.is_match("/src/rules/read.rs"));
    }

    #[test]
    fn double_star_ext_deep_nested() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/src/**/*.rs");
        assert!(rule.is_match("/src/a/b/c/lib.rs"));
    }

    #[test]
    fn double_star_ext_wrong_extension() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/src/**/*.rs");
        assert!(!rule.is_match("/src/rules/read.toml"));
    }

    #[test]
    fn exact_path_same() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/etc/hosts");
        assert!(rule.is_match("/etc/hosts"));
    }

    #[test]
    fn exact_path_different() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/etc/hosts");
        assert!(!rule.is_match("/etc/passwd"));
    }

    #[test]
    fn exact_path_nested() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("/etc/hosts");
        assert!(!rule.is_match("/etc/hosts/extra"));
    }

    #[test]
    fn bare_filename_anywhere() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("CLAUDE.md");
        assert!(rule.is_match("/home/user/project/.claude/CLAUDE.md"));
        assert!(rule.is_match("/tmp/CLAUDE.md"));
    }

    #[test]
    fn bare_filename_different_name() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("CLAUDE.md");
        assert!(!rule.is_match("/home/user/README.md"));
    }

    #[test]
    fn bare_glob_basename() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("*.md");
        assert!(rule.is_match("/home/user/project/README.md"));
        assert!(rule.is_match("/tmp/CLAUDE.md"));
    }

    #[test]
    fn bare_glob_wrong_extension() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create("*.md");
        assert!(!rule.is_match("/home/user/project/lib.rs"));
    }

    #[test]
    fn bare_dotfile_pattern() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create(".env");
        assert!(rule.is_match("/home/user/project/.env"));
    }

    #[test]
    fn bare_dotfile_glob() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create(".env.*");
        assert!(rule.is_match("/home/user/project/.env.local"));
        assert!(rule.is_match("/tmp/.env.production"));
    }

    #[test]
    fn bare_dotfile_glob_bare_env() {
        let factory = ServiceBuilder::mock().build().expect::<PathRuleFactory>();
        let rule = factory.create(".env.*");
        assert!(!rule.is_match("/home/user/project/.env"));
    }
}
