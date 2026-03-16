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

    /// Compile a pattern into a [`PathRule`] with tilde expansion.
    pub fn create(&self, pattern: impl Into<String>) -> PathRule {
        let pattern = expand_tilde(pattern, &self.home);
        let Some(matcher) = compile_path_glob(&pattern) else {
            return PathRule::new(Some(pattern), None);
        };
        let exact = strip_recursive_glob(&pattern);
        PathRule::new(exact, Some(matcher))
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

/// Expand a leading `~/` to the given home directory.
fn expand_tilde(pattern: impl Into<String>, home: &Path) -> String {
    let pattern = pattern.into();
    if let Some(rest) = pattern.strip_prefix("~/") {
        return format!("{}/{rest}", home.to_string_lossy());
    }
    if pattern.contains('~') {
        warn!("Pattern contains unexpected tilde: {pattern}");
    }
    pattern
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
    fn bare_tilde_unchanged() {
        let result = expand_tilde("~", &home());
        assert_eq!(result, "~");
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
    fn bare_tilde_no_glob_match() {
        let rule = factory().create("~");
        assert!(!rule.is_match("/home/user"));
    }

    #[test]
    fn tilde_other_user_no_glob_match() {
        let rule = factory().create("~other/file");
        assert!(!rule.is_match("/home/other/file"));
    }
}
