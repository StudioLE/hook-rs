//! Evaluation of Read tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate Read tool calls against trusted path rules.
pub struct ReadHandler;

impl Handler for ReadHandler {
    type Input = ReadInput;

    fn run(input: Self::Input, settings: Settings) -> Option<Outcome> {
        trace!(file_path = %input.file_path, "Handling read path");
        let home =
            dirs::home_dir().expect("home directory should be resolvable via $HOME or passwd");
        let rules: Vec<ReadRule> = RuleFactory::new(settings.read.paths.clone(), home).create();
        rules
            .iter()
            .find(|rule| rule.matches(&input.file_path))
            .map(|rule| rule.outcome.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_path_allowed() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/data/file.txt");
        let settings = absolute_settings();

        // Act
        let outcome = ReadHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn second_pattern_allowed() {
        // Arrange
        let input = ReadInput::new("/usr/share/doc/rust/html/index.html");
        let settings = absolute_settings();

        // Act
        let outcome = ReadHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_path_no_match() {
        // Arrange
        let input = ReadInput::new("/etc/passwd");
        let settings = absolute_settings();

        // Act
        let outcome = ReadHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings_no_match() {
        // Arrange
        let input = ReadInput::new("/opt/readonly/file.txt");
        let settings = Settings::default();

        // Act
        let outcome = ReadHandler::run(input, settings);

        // Assert
        assert!(outcome.is_none());
    }

    #[test]
    fn tilde_pattern_expands_to_real_home() {
        // Arrange
        let home = dirs::home_dir().expect("test requires home directory");
        let input = ReadInput::new(format!(
            "{home}/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs",
            home = home.display()
        ));
        let settings = Settings {
            read: ReadSettings {
                paths: vec!["~/.cargo/registry/src/**".to_owned()],
            },
            ..Settings::default()
        };

        // Act
        let outcome = ReadHandler::run(input, settings);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    fn absolute_settings() -> Settings {
        Settings {
            read: ReadSettings {
                paths: vec![
                    "/opt/readonly/**".to_owned(),
                    "/usr/share/doc/**".to_owned(),
                ],
            },
            ..Settings::default()
        }
    }
}
