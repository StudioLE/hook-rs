//! Evaluation of Read tool calls against trusted path rules.

use crate::prelude::*;

/// Evaluate a file path against all read rules, returning the first match.
pub fn evaluate_read(settings: &Settings, file_path: &str) -> Option<Outcome> {
    let home = dirs::home_dir().expect("home directory should be resolvable via $HOME or passwd");
    ReadRuleFactory::new(settings.read.paths.clone(), home)
        .create()
        .iter()
        .find(|rule| rule.matches(file_path))
        .map(|rule| rule.outcome.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn matching_path_allowed() {
        let outcome = evaluate_read(&absolute_settings(), "/opt/readonly/data/file.txt");
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn second_pattern_allowed() {
        let outcome = evaluate_read(&absolute_settings(), "/usr/share/doc/rust/html/index.html");
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }

    #[test]
    fn unrelated_path_no_match() {
        let outcome = evaluate_read(&absolute_settings(), "/etc/passwd");
        assert!(outcome.is_none());
    }

    #[test]
    fn empty_settings_no_match() {
        let outcome = evaluate_read(&Settings::default(), "/opt/readonly/file.txt");
        assert!(outcome.is_none());
    }

    #[test]
    fn tilde_pattern_expands_to_real_home() {
        let home = dirs::home_dir().expect("test requires home directory");
        let settings = Settings {
            read: ReadSettings {
                paths: vec!["~/.cargo/registry/src/**".to_owned()],
            },
            ..Settings::default()
        };
        let path = format!(
            "{home}/.cargo/registry/src/index.crates.io-xxx/serde-1.0.0/src/lib.rs",
            home = home.display()
        );
        assert_eq!(
            evaluate_read(&settings, &path)
                .expect("should match")
                .decision,
            Decision::Allow
        );
    }
}
