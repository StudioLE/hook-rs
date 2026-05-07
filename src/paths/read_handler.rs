//! Evaluation of Read tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate Read tool calls against trusted path rules.
#[derive(FromServices)]
pub struct ReadHandler {
    /// Factory for building path-matching rules.
    path_rule_factory: Arc<PathRuleFactory>,
    /// User settings containing trusted path patterns.
    settings: Arc<Settings>,
}

impl Handler for ReadHandler {
    type Input = ReadInput;

    fn run(&self, input: Self::Input) -> Option<Outcome> {
        trace!(path = %input.file_path, "Handling read");
        self.path_rule_factory
            .is_match_outcome(&input.file_path, &self.settings.read.paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_path() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/data/file.txt");

        // Act
        let outcome = handler(Settings::with_read(&[
            "/opt/readonly/**",
            "/usr/share/doc/**",
        ]))
        .run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn second_pattern_match() {
        // Arrange
        let input = ReadInput::new("/usr/share/doc/rust/html/index.html");

        // Act
        let outcome = handler(Settings::with_read(&[
            "/opt/readonly/**",
            "/usr/share/doc/**",
        ]))
        .run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_path() {
        // Arrange
        let input = ReadInput::new("/etc/passwd");

        // Act
        let outcome = handler(Settings::with_read(&[
            "/opt/readonly/**",
            "/usr/share/doc/**",
        ]))
        .run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/file.txt");

        // Act
        let outcome = handler(Settings::default()).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn tilde_pattern_expands_to_mock_home() {
        // Arrange
        let input = ReadInput::new(
            "/home/user/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs",
        );
        let settings = Settings::with_read(&["~/.cargo/registry/src/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn negation_excludes_path() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/secret/key.pem");
        let settings = Settings::with_read(&["/opt/readonly/**", "!/opt/readonly/secret/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn re_include_after_negation() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/secret/public.txt");
        let settings = Settings::with_read(&[
            "/opt/readonly/**",
            "!/opt/readonly/secret/**",
            "/opt/readonly/secret/public.txt",
        ]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    /// Tilde input path matches a tilde settings pattern via `PathRuleFactory` expansion.
    #[test]
    fn tilde_input_with_tilde_pattern() {
        // Arrange
        let input =
            ReadInput::new("~/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs");
        let settings = Settings::with_read(&["~/.cargo/registry/src/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    /// Tilde input path matches an absolute settings pattern.
    #[test]
    fn tilde_input_with_absolute_pattern() {
        // Arrange
        let input =
            ReadInput::new("~/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs");
        let settings = Settings::with_read(&["/home/user/.cargo/registry/src/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    /// Tilde input path to a deeply nested file.
    #[test]
    fn tilde_input_deep_path() {
        // Arrange
        let input = ReadInput::new("~/.config/tools/cache/v1/data/file.md");
        let settings = Settings::with_read(&["~/.config/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    fn handler(settings: Settings) -> ReadHandler {
        ReadHandler {
            path_rule_factory: Arc::new(PathRuleFactory::mock()),
            settings: Arc::new(settings),
        }
    }
}
