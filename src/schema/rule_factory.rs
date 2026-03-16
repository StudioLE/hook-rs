//! Generic factory for building glob-based rules with tilde expansion.

use crate::prelude::*;
use globset::GlobBuilder;
use globset::GlobMatcher;
use std::path::Path;
use std::path::PathBuf;

/// Construct a rule from a compiled glob matcher, expanded pattern, and outcome.
pub trait FromGlob {
    /// Create a rule from glob compilation results.
    fn from_glob(matcher: GlobMatcher, pattern: String, outcome: Outcome) -> Self;
}

/// Build glob-based rules from patterns, expanding `~/` to a concrete home directory.
pub struct RuleFactory {
    /// Raw glob patterns before expansion.
    patterns: Vec<String>,
    /// Home directory for tilde expansion.
    home: PathBuf,
}

impl RuleFactory {
    /// Create a new [`RuleFactory`] from raw glob patterns and a home directory.
    pub fn new(patterns: Vec<String>, home: PathBuf) -> Self {
        Self { patterns, home }
    }

    /// Compile each pattern into a rule, skipping invalid patterns.
    pub fn create<T: FromGlob>(&self) -> Vec<T> {
        self.patterns
            .iter()
            .filter_map(|pat| {
                let expanded = expand_tilde(pat, &self.home)?;
                let matcher = GlobBuilder::new(&expanded)
                    .literal_separator(true)
                    .build()
                    .ok()?
                    .compile_matcher();
                Some(T::from_glob(
                    matcher,
                    expanded,
                    Outcome::allow(format!("Safe read-only path: {pat}")),
                ))
            })
            .collect()
    }
}

/// Expand a leading `~/` to the given home directory.
///
/// Returns `None` if the pattern contains `~` in any other position.
fn expand_tilde(pattern: &str, home: &Path) -> Option<String> {
    if let Some(rest) = pattern.strip_prefix("~/") {
        let home = home.to_str()?;
        Some(format!("{home}/{rest}"))
    } else if pattern.contains('~') {
        None
    } else {
        Some(pattern.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn home() -> PathBuf {
        PathBuf::from("/home/user")
    }

    // expand_tilde

    #[test]
    fn tilde_prefix_expands() {
        let result = expand_tilde("~/.cargo/**", &home());
        assert_eq!(result.expect("should expand"), "/home/user/.cargo/**");
    }

    #[test]
    fn tilde_in_middle_returns_none() {
        assert!(expand_tilde("/tmp/~backup/file", &home()).is_none());
    }

    #[test]
    fn bare_tilde_returns_none() {
        assert!(expand_tilde("~", &home()).is_none());
    }

    #[test]
    fn tilde_other_user_returns_none() {
        assert!(expand_tilde("~other/file", &home()).is_none());
    }

    #[test]
    fn no_tilde_passes_through() {
        let result = expand_tilde("/absolute/path/**", &home());
        assert_eq!(result.expect("should pass through"), "/absolute/path/**");
    }

    // integration: tilde expansion + glob matching through factory

    fn factory(patterns: &[&str]) -> RuleFactory {
        RuleFactory::new(patterns.iter().map(|s| (*s).to_owned()).collect(), home())
    }

    fn matches_any(rules: &[ReadRule], path: &str) -> bool {
        rules.iter().any(|r| r.matches(path))
    }

    #[test]
    fn cargo_registry_source_matches() {
        let rules: Vec<ReadRule> = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(matches_any(
            &rules,
            "/home/user/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs"
        ));
    }

    #[test]
    fn cargo_registry_nested_matches() {
        let rules: Vec<ReadRule> = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(matches_any(
            &rules,
            "/home/user/.cargo/registry/src/index.crates.io-xxx/deep/nested/file.rs"
        ));
    }

    #[test]
    fn rustup_toolchain_matches() {
        let rules: Vec<ReadRule> = factory(&["~/.rustup/toolchains/**"]).create();
        assert!(matches_any(
            &rules,
            "/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/lib.rs"
        ));
    }

    #[test]
    fn unrelated_path_no_match() {
        let rules: Vec<ReadRule> =
            factory(&["~/.cargo/registry/src/**", "~/.rustup/toolchains/**"]).create();
        assert!(!matches_any(&rules, "/etc/passwd"));
    }

    #[test]
    fn home_prefix_mismatch() {
        let rules: Vec<ReadRule> = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(!matches_any(
            &rules,
            "/home/other/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs"
        ));
    }

    #[test]
    fn cargo_registry_root_no_match() {
        let rules: Vec<ReadRule> = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(!matches_any(
            &rules,
            "/home/user/.cargo/registry/cache/something"
        ));
    }

    #[test]
    fn absolute_path_without_tilde() {
        let rules: Vec<ReadRule> = factory(&["/opt/readonly/**"]).create();
        assert!(matches_any(&rules, "/opt/readonly/foo/bar.rs"));
        assert!(!matches_any(&rules, "/opt/other/foo.rs"));
    }

    #[test]
    fn tilde_in_middle_skipped() {
        let rules: Vec<ReadRule> = factory(&["/tmp/~backup/file"]).create();
        assert!(rules.is_empty());
    }

    #[test]
    fn bare_tilde_skipped() {
        let rules: Vec<ReadRule> = factory(&["~"]).create();
        assert!(rules.is_empty());
    }

    #[test]
    fn tilde_user_skipped() {
        let rules: Vec<ReadRule> = factory(&["~other/file"]).create();
        assert!(rules.is_empty());
    }
}
