//! Factory for building [`ReadRule`] instances with tilde expansion.

use crate::prelude::*;
use globset::GlobBuilder;
use std::path::PathBuf;

/// Build [`ReadRule`] instances from glob patterns, expanding `~/` to a concrete home directory.
pub struct ReadRuleFactory {
    patterns: Vec<String>,
    home: PathBuf,
}

impl ReadRuleFactory {
    /// Create a new [`ReadRuleFactory`] from raw glob patterns and a home directory for tilde expansion.
    pub(crate) fn new(patterns: Vec<String>, home: PathBuf) -> Self {
        Self { patterns, home }
    }

    /// Compile each pattern into a [`ReadRule`], skipping invalid patterns.
    pub(crate) fn create(&self) -> Vec<ReadRule> {
        self.patterns
            .iter()
            .filter_map(|pat| {
                let expanded = self.expand_tilde(pat)?;
                let matcher = GlobBuilder::new(&expanded)
                    .literal_separator(true)
                    .build()
                    .ok()?
                    .compile_matcher();
                Some(ReadRule::new(
                    matcher,
                    Outcome::allow(format!("Safe read-only path: {pat}")),
                ))
            })
            .collect()
    }

    /// Expand a leading `~/` to `self.home`.
    ///
    /// Returns `None` if the pattern contains `~` in any other position.
    fn expand_tilde(&self, pattern: &str) -> Option<String> {
        if let Some(rest) = pattern.strip_prefix("~/") {
            let home = self.home.to_str()?;
            Some(format!("{home}/{rest}"))
        } else if pattern.contains('~') {
            None
        } else {
            Some(pattern.to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factory(patterns: &[&str]) -> ReadRuleFactory {
        ReadRuleFactory::new(
            patterns.iter().map(|s| (*s).to_owned()).collect(),
            PathBuf::from("/home/user"),
        )
    }

    fn matches_any(rules: &[ReadRule], path: &str) -> bool {
        rules.iter().any(|r| r.matches(path))
    }

    // expand_tilde

    #[test]
    fn tilde_expands_to_home() {
        let rules = factory(&["~/.cargo/**"]).create();
        assert!(matches_any(&rules, "/home/user/.cargo/registry/src/lib.rs"));
    }

    #[test]
    fn tilde_in_middle_skipped() {
        let rules = factory(&["/tmp/~backup/file"]).create();
        assert!(rules.is_empty());
    }

    #[test]
    fn bare_tilde_skipped() {
        let rules = factory(&["~"]).create();
        assert!(rules.is_empty());
    }

    #[test]
    fn tilde_user_skipped() {
        let rules = factory(&["~other/file"]).create();
        assert!(rules.is_empty());
    }

    #[test]
    fn no_tilde_passes_through() {
        let rules = factory(&["/absolute/path/**"]).create();
        assert!(matches_any(&rules, "/absolute/path/foo.rs"));
    }

    // integration: real expansion + glob matching

    #[test]
    fn cargo_registry_source_matches() {
        let rules = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(matches_any(
            &rules,
            "/home/user/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs"
        ));
    }

    #[test]
    fn cargo_registry_nested_matches() {
        let rules = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(matches_any(
            &rules,
            "/home/user/.cargo/registry/src/index.crates.io-xxx/deep/nested/file.rs"
        ));
    }

    #[test]
    fn rustup_toolchain_matches() {
        let rules = factory(&["~/.rustup/toolchains/**"]).create();
        assert!(matches_any(
            &rules,
            "/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/lib.rs"
        ));
    }

    #[test]
    fn unrelated_path_no_match() {
        let rules = factory(&["~/.cargo/registry/src/**", "~/.rustup/toolchains/**"]).create();
        assert!(!matches_any(&rules, "/etc/passwd"));
    }

    #[test]
    fn home_prefix_mismatch() {
        let rules = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(!matches_any(
            &rules,
            "/home/other/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs"
        ));
    }

    #[test]
    fn cargo_registry_root_no_match() {
        let rules = factory(&["~/.cargo/registry/src/**"]).create();
        assert!(!matches_any(
            &rules,
            "/home/user/.cargo/registry/cache/something"
        ));
    }

    #[test]
    fn absolute_path_without_tilde() {
        let rules = factory(&["/opt/readonly/**"]).create();
        assert!(matches_any(&rules, "/opt/readonly/foo/bar.rs"));
        assert!(!matches_any(&rules, "/opt/other/foo.rs"));
    }
}
