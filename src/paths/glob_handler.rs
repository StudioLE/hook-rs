//! Evaluation of Glob tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate Glob tool calls against trusted path rules.
#[derive(FromServices)]
pub struct GlobHandler {
    /// Factory for building path-matching rules.
    path_rule_factory: Arc<PathRuleFactory>,
    /// User settings containing trusted path patterns.
    settings: Arc<Settings>,
}

impl Handler for GlobHandler {
    type Input = GlobInput;

    fn run(&self, input: Self::Input) -> Option<Outcome> {
        let path = input.path.unwrap_or_cwd();
        trace!(path, "Handling glob");
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
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly".to_owned()));

        // Act
        let outcome = handler(Settings::with_read(&["/opt/readonly/**"])).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn file_path_directly() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly/src/lib.rs".to_owned()));

        // Act
        let outcome = handler(Settings::with_read(&["/opt/readonly/**"])).run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_directory() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/etc".to_owned()));

        // Act
        let outcome = handler(Settings::with_read(&["/opt/readonly/**"])).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly".to_owned()));

        // Act
        let outcome = handler(Settings::default()).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn missing_path_falls_back_to_cwd() {
        // Arrange
        let input = GlobInput::new("**/*.rs", None);
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
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly/secret".to_owned()));
        let settings = Settings::with_read(&["/opt/readonly/**", "!/opt/readonly/secret/**"]);

        // Act
        let outcome = handler(settings).run(input);

        // Assert
        assert!(outcome.is_none());
    }

    fn handler(settings: Settings) -> GlobHandler {
        GlobHandler {
            path_rule_factory: Arc::new(PathRuleFactory::mock()),
            settings: Arc::new(settings),
        }
    }
}
