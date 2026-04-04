//! Factory for building glob-based [`PathRule`] with tilde expansion.

use crate::prelude::*;

/// Build glob-based rules from patterns, expanding `~/` to a concrete home directory.
pub struct PathRuleFactory {
    /// Home directory for tilde expansion.
    home: PathBuf,
}

impl PathRuleFactory {
    /// Create a new [`PathRuleFactory`] with the given home directory for tilde expansion.
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    /// Compile a pattern into a [`PathRule`].
    ///
    /// - Expands a leading `~/` to the home directory
    /// - Patterns without `/` match against the filename component only
    pub fn create(&self, pattern: impl Into<String>) -> PathRule {
        let pattern = expand_tilde(pattern, &self.home);
        let is_filename = !pattern.contains('/');
        let Some(matcher) = compile_path_glob(&pattern) else {
            return PathRule::new(Some(pattern), None, is_filename);
        };
        let exact = strip_recursive_glob(&pattern);
        PathRule::new(exact, Some(matcher), is_filename)
    }

    /// Check if a path is allowed using last-match-wins semantics.
    ///
    /// - Expands a leading `~/` or bare `~` in the input path
    /// - Patterns are evaluated bottom-to-top (last match wins)
    /// - `!` prefix negates (untrusts)
    /// - No match returns `None`
    pub fn is_match(&self, path: &str, patterns: &[String]) -> Option<bool> {
        let path = expand_tilde(path, &self.home);
        for pattern in patterns.iter().rev() {
            let (negated, glob) = match pattern.strip_prefix('!') {
                Some(rest) => (true, rest),
                None => (false, pattern.as_str()),
            };
            if self.create(glob).is_match(&path) {
                return Some(!negated);
            }
        }
        None
    }

    /// Check if a path is allowed and return an [`Outcome`] for allowed paths.
    ///
    /// - Returns `Some(Outcome::allow(...))` if the path matches a non-negated pattern
    /// - Returns `None` if no match or negated (passthrough to default permissions)
    pub fn is_match_outcome(&self, path: &str, patterns: &[String]) -> Option<Outcome> {
        if let Some(is_allowed) = self.is_match(path, patterns) {
            trace!(is_allowed, "Matched");
            if is_allowed {
                return Some(Outcome::allow("Path is allowed"));
            }
        } else {
            trace!("No match");
        }
        None
    }
}

impl Default for PathRuleFactory {
    fn default() -> Self {
        let home = dirs::home_dir().expect("home directory should be resolvable");
        Self::new(home)
    }
}

/// Strip `/**` or `/**/*` suffix, returning the base directory for exact matching.
fn strip_recursive_glob(pattern: &str) -> Option<String> {
    pattern
        .strip_suffix("/**/*")
        .or_else(|| pattern.strip_suffix("/**"))
        .map(ToOwned::to_owned)
}

/// Expand a leading `~/` or bare `~` to the given home directory.
fn expand_tilde(value: impl Into<String>, home: &Path) -> String {
    let value = value.into();
    if value == "~" {
        return home.to_string_lossy().into_owned();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return format!("{}/{rest}", home.to_string_lossy());
    }
    if value.contains('~') {
        warn!("Path contains unexpected tilde: {value}");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> PathBuf {
        PathBuf::from("/home/user")
    }

    fn factory() -> PathRuleFactory {
        PathRuleFactory::new(home())
    }

    // expand_tilde

    #[test]
    fn tilde_prefix_expands() {
        let result = expand_tilde("~/.cargo/**", &home());
        assert_eq!(result, "/home/user/.cargo/**");
    }

    #[test]
    fn tilde_in_middle_unchanged() {
        let result = expand_tilde("/tmp/~backup/file", &home());
        assert_eq!(result, "/tmp/~backup/file");
    }

    #[test]
    fn bare_tilde() {
        let output = expand_tilde("~", &home());
        assert_eq!(output, "/home/user");
    }

    #[test]
    fn tilde_other_user_unchanged() {
        let result = expand_tilde("~other/file", &home());
        assert_eq!(result, "~other/file");
    }

    #[test]
    fn no_tilde_passes_through() {
        let result = expand_tilde("/absolute/path/**", &home());
        assert_eq!(result, "/absolute/path/**");
    }

    // integration: tilde expansion + glob matching through factory

    #[test]
    fn cargo_registry_source_matches() {
        let rule = factory().create("~/.cargo/registry/src/**");
        assert!(
            rule.is_match(
                "/home/user/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs"
            )
        );
    }

    #[test]
    fn cargo_registry_nested_matches() {
        let rule = factory().create("~/.cargo/registry/src/**");
        assert!(
            rule.is_match("/home/user/.cargo/registry/src/index.crates.io-xxx/deep/nested/file.rs")
        );
    }

    #[test]
    fn rustup_toolchain_matches() {
        let rule = factory().create("~/.rustup/toolchains/**");
        assert!(rule.is_match(
            "/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/lib.rs"
        ));
    }

    #[test]
    fn unrelated_path_no_match() {
        let cargo = factory().create("~/.cargo/registry/src/**");
        let rustup = factory().create("~/.rustup/toolchains/**");
        assert!(!cargo.is_match("/etc/passwd"));
        assert!(!rustup.is_match("/etc/passwd"));
    }

    #[test]
    fn home_prefix_mismatch() {
        let rule = factory().create("~/.cargo/registry/src/**");
        assert!(!rule.is_match(
            "/home/other/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs"
        ));
    }

    #[test]
    fn cargo_registry_root_no_match() {
        let rule = factory().create("~/.cargo/registry/src/**");
        assert!(!rule.is_match("/home/user/.cargo/registry/cache/something"));
    }

    #[test]
    fn absolute_path_without_tilde() {
        let rule = factory().create("/opt/readonly/**");
        assert!(rule.is_match("/opt/readonly/foo/bar.rs"));
        assert!(!rule.is_match("/opt/other/foo.rs"));
    }

    #[test]
    fn tilde_in_middle_no_glob_match() {
        let rule = factory().create("/tmp/~backup/file");
        assert!(!rule.is_match("/tmp/something"));
    }

    #[test]
    fn bare_tilde_pattern() {
        let rule = factory().create("~");
        assert!(rule.is_match("/home/user"));
        assert!(!rule.is_match("/home/user/subdir"));
    }

    #[test]
    fn tilde_other_user_no_glob_match() {
        let rule = factory().create("~other/file");
        assert!(!rule.is_match("/home/other/file"));
    }

    // is_match with patterns

    #[test]
    fn patterns_simple_match() {
        let patterns = vec!["/a/**".to_owned()];
        assert_eq!(factory().is_match("/a/file.txt", &patterns), Some(true));
    }

    #[test]
    fn patterns_no_match() {
        let patterns = vec!["/a/**".to_owned()];
        assert_eq!(factory().is_match("/b/file.txt", &patterns), None);
    }

    #[test]
    fn patterns_negation_excludes() {
        let patterns = vec!["/a/**".to_owned(), "!/a/secret/**".to_owned()];
        assert_eq!(
            factory().is_match("/a/secret/key.pem", &patterns),
            Some(false)
        );
    }

    #[test]
    fn patterns_re_include_after_negation() {
        let patterns = vec![
            "/a/**".to_owned(),
            "!/a/secret/**".to_owned(),
            "/a/secret/public.txt".to_owned(),
        ];
        assert_eq!(
            factory().is_match("/a/secret/public.txt", &patterns),
            Some(true)
        );
    }

    #[test]
    fn patterns_last_match_wins() {
        let patterns = vec!["!/a/**".to_owned(), "/a/**".to_owned()];
        assert_eq!(factory().is_match("/a/file.txt", &patterns), Some(true));
    }

    #[test]
    fn patterns_empty() {
        let patterns: Vec<String> = vec![];
        assert_eq!(factory().is_match("/a/file.txt", &patterns), None);
    }

    // is_match with tilde input paths

    /// Input path with `~` matches a tilde settings pattern.
    #[test]
    fn is_match__tilde_input_tilde_pattern() {
        let patterns = vec!["~/.config/foo/**".to_owned()];
        assert_eq!(
            factory().is_match("~/.config/foo/bar", &patterns),
            Some(true)
        );
    }

    /// Input path with `~` matches an absolute settings pattern.
    #[test]
    fn is_match__tilde_input_absolute_pattern() {
        let patterns = vec!["/home/user/.config/foo/**".to_owned()];
        assert_eq!(
            factory().is_match("~/.config/foo/bar", &patterns),
            Some(true)
        );
    }

    /// Already-expanded input path matches a tilde settings pattern.
    #[test]
    fn is_match__absolute_input_tilde_pattern() {
        let patterns = vec!["~/.config/foo/**".to_owned()];
        assert_eq!(
            factory().is_match("/home/user/.config/foo/bar", &patterns),
            Some(true)
        );
    }

    /// Bare `~` input matches a `~/**` pattern via exact prefix.
    #[test]
    fn is_match__bare_tilde_input() {
        let patterns = vec!["~/**".to_owned()];
        assert_eq!(factory().is_match("~", &patterns), Some(true));
    }

    /// Trailing-slash tilde input matches `~/**` pattern.
    #[test]
    fn is_match__tilde_slash_input() {
        let patterns = vec!["~/**".to_owned()];
        assert_eq!(factory().is_match("~/", &patterns), Some(true));
    }

    /// Tilde input with negation pattern.
    #[test]
    fn is_match__tilde_input_negation() {
        let patterns = vec!["~/.config/**".to_owned(), "!~/.config/secret/**".to_owned()];
        assert_eq!(
            factory().is_match("~/.config/secret/key", &patterns),
            Some(false)
        );
    }

    /// Already-expanded path is not corrupted by `expand_tilde`.
    #[test]
    fn is_match__no_double_expansion() {
        let patterns = vec!["/home/user/.config/**".to_owned()];
        assert_eq!(
            factory().is_match("/home/user/.config/foo", &patterns),
            Some(true)
        );
    }

    // basename patterns

    #[test]
    fn patterns_bare_filename_matches() {
        let patterns = vec!["CLAUDE.md".to_owned()];
        assert_eq!(
            factory().is_match("/home/user/project/.claude/CLAUDE.md", &patterns),
            Some(true)
        );
    }

    #[test]
    fn patterns_bare_filename_negation() {
        let patterns = vec!["CLAUDE.md".to_owned(), "!.env".to_owned()];
        assert_eq!(
            factory().is_match("/home/user/project/.env", &patterns),
            Some(false)
        );
    }

    #[test]
    fn patterns_bare_glob_matches() {
        let patterns = vec![".env.*".to_owned()];
        assert_eq!(
            factory().is_match("/home/user/project/.env.local", &patterns),
            Some(true)
        );
    }
}
