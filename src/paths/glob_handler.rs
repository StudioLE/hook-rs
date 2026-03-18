//! Evaluation of Glob tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate Glob tool calls against trusted path rules.
pub struct GlobHandler;

impl Handler for GlobHandler {
    type Input = GlobInput;

    fn run(input: Self::Input, settings: Settings) -> Option<Outcome> {
        let path = input.path.unwrap_or_cwd();
        trace!(path, "Handling glob");
        let factory = PathRuleFactory::default();
        factory.is_match_outcome(&path, &settings.read.paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_matches_via_prefix() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly".to_owned()));
        let settings = settings();

        // Act
        let outcome = GlobHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn file_path_matches_directly() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly/src/lib.rs".to_owned()));
        let settings = settings();

        // Act
        let outcome = GlobHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_directory_no_match() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/etc".to_owned()));
        let settings = settings();

        // Act
        let outcome = GlobHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings_no_match() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly".to_owned()));
        let settings = Settings::default();

        // Act
        let outcome = GlobHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn missing_path_falls_back_to_cwd() {
        // Arrange
        let input = GlobInput::new("**/*.rs", None);
        let cwd = cwd();
        let settings = Settings {
            read: ReadSettings {
                paths: vec![format!("{cwd}/**")],
            },
            ..Settings::default()
        };

        // Act
        let outcome = GlobHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn negation_excludes_path() {
        // Arrange
        let input = GlobInput::new("**/*.rs", Some("/opt/readonly/secret".to_owned()));
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
        let outcome = GlobHandler::run(input, settings);

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
