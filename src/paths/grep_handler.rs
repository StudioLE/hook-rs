//! Evaluation of Grep tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate Grep tool calls against trusted path rules.
#[derive(FromServices)]
pub struct GrepHandler {
    /// Factory for building path-matching rules.
    path_rule_factory: Arc<PathRuleFactory>,
    /// User settings containing trusted path patterns.
    settings: Arc<Settings>,
}

impl Handler for GrepHandler {
    type Input = GrepInput;

    fn run(&self, input: Self::Input) -> Option<Outcome> {
        let path = input.path.unwrap_or_cwd();
        trace!(path, "Handling grep");
        self.path_rule_factory
            .is_match_outcome(&path, &self.settings.read.paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_via_prefix() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly");

        // Act
        let outcome = handler(Settings::with_read(&["/opt/readonly/**"])).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn file_path_directly() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly/src/lib.rs");

        // Act
        let outcome = handler(Settings::with_read(&["/opt/readonly/**"])).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_directory() {
        // Arrange
        let input = GrepInput::new("needle", "/etc");

        // Act
        let outcome = handler(Settings::with_read(&["/opt/readonly/**"])).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly");

        // Act
        let outcome = handler(Settings::default()).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn missing_path_falls_back_to_cwd() {
        // Arrange
        let input = GrepInput {
            pattern: "needle".to_owned(),
            path: None,
        };
        let cwd = cwd();
        let settings = Settings::with_read(&[&format!("{cwd}/**")]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn negation_excludes_path() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly/secret");
        let settings = Settings::with_read(&["/opt/readonly/**", "!/opt/readonly/secret/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    fn handler(settings: Settings) -> GrepHandler {
        GrepHandler {
            path_rule_factory: Arc::new(PathRuleFactory::mock()),
            settings: Arc::new(settings),
        }
    }
}
