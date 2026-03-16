//! Evaluation of Grep tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate Grep tool calls against trusted path rules.
pub struct GrepHandler;

impl Handler for GrepHandler {
    type Input = GrepInput;

    fn run(input: Self::Input, settings: Settings) -> Option<Outcome> {
        trace!(path = %input.path, "Handling grep");
        let factory = PathRuleFactory::default();
        factory.is_match_outcome(&input.path, &settings.read.paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_matches_via_prefix() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly");
        let settings = settings();

        // Act
        let outcome = GrepHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn file_path_matches_directly() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly/src/lib.rs");
        let settings = settings();

        // Act
        let outcome = GrepHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_directory_no_match() {
        // Arrange
        let input = GrepInput::new("needle", "/etc");
        let settings = settings();

        // Act
        let outcome = GrepHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings_no_match() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly");
        let settings = Settings::default();

        // Act
        let outcome = GrepHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn negation_excludes_path() {
        // Arrange
        let input = GrepInput::new("needle", "/opt/readonly/secret");
        let settings = Settings {
            read: ReadSettings {
                paths: vec![
                    "/opt/readonly/**".to_owned(),
                    "!/opt/readonly/secret/**".to_owned(),
                ],
            },
            ..Settings::default()
        };

        // Act
        let outcome = GrepHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    fn settings() -> Settings {
        Settings {
            read: ReadSettings {
                paths: vec!["/opt/readonly/**".to_owned()],
            },
            ..Settings::default()
        }
    }
}
