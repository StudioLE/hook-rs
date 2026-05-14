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
        let settings = Settings::with_read(&["/opt/readonly/**", "/usr/share/doc/**"]);
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn second_pattern_match() {
        // Arrange
        let input = ReadInput::new("/usr/share/doc/rust/html/index.html");
        let settings = Settings::with_read(&["/opt/readonly/**", "/usr/share/doc/**"]);
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_path() {
        // Arrange
        let input = ReadInput::new("/etc/passwd");
        let settings = Settings::with_read(&["/opt/readonly/**", "/usr/share/doc/**"]);
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/file.txt");
        let settings = Settings::default();
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

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
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn negation_excludes_path() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/secret/key.pem");
        let settings = Settings::with_read(&["/opt/readonly/**", "!/opt/readonly/secret/**"]);
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

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
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

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
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

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
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    /// Tilde input path to a deeply nested file.
    #[test]
    fn tilde_input_deep_path() {
        // Arrange
        let input = ReadInput::new("~/.config/tools/cache/v1/data/file.md");
        let settings = Settings::with_read(&["~/.config/**"]);
        let handler = ServiceBuilder::mock()
            .with_instance(settings)
            .build()
            .expect::<ReadHandler>();

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }
}
