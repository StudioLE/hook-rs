//! User-specific settings loaded from `~/.config/claude-hooks/settings.yaml`.

use crate::prelude::*;
use std::fs;

/// User-specific settings for rule evaluation.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Settings {
    /// Git-specific settings for `git -C` path classification.
    #[serde(default)]
    pub git: GitSettings,
    /// Read tool settings for auto-allowing trusted file paths.
    #[serde(default)]
    pub read: ReadSettings,
}

/// Glob patterns for auto-allowing Read tool access to trusted paths.
///
/// Patterns starting with `~/` are expanded to `$HOME/`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ReadSettings {
    /// Glob patterns for paths that are safe to read without prompting.
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Git path classification for `git -C` operations.
///
/// Ordered glob patterns following `.gitignore` semantics:
///
/// - Evaluated top-to-bottom, last match wins
/// - Prefix with `!` to negate (untrust)
/// - Paths matching no pattern are untrusted
/// - Supports tilde expansion (`~/repos/**`)
///
/// ```yaml
/// git:
///   paths:
///     - /home/user/repos/**
///     - !/home/user/repos/forked/**
///     - /home/user/repos/forked/this
/// ```
///
/// See CVE-2025-59536 and CVE-2026-21852.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GitSettings {
    /// Glob patterns for `git -C` trust classification.
    ///
    /// - Last matching pattern wins
    /// - Prefix with `!` to negate
    /// - Supports tilde expansion (`~/repos/**`)
    #[serde(default)]
    pub paths: Vec<String>,
}

impl Settings {
    /// Load settings from file.
    ///
    /// Returns `Settings::default()` if the file is missing.
    pub fn load() -> Result<Self, Report<SettingsError>> {
        let path = config_path();
        if !path.exists() {
            warn!(path = %path.display(), "Settings file not found");
            debug!("Using default settings");
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).change_context(SettingsError::Read)?;
        let yaml = quote_yaml_tags(&raw);
        let settings: Settings =
            serde_yaml::from_str(&yaml).change_context(SettingsError::Deserialize)?;
        trace!(
            path = %path.display(),
            git_paths = settings.git.paths.len(),
            read_paths = settings.read.paths.len(),
            "Loaded settings",
        );
        Ok(settings)
    }

    /// Mock settings for use in tests.
    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            git: GitSettings {
                paths: vec![
                    "/home/user/repos/**".to_owned(),
                    "!/home/user/repos/forked/**".to_owned(),
                ],
            },
            read: ReadSettings {
                paths: vec![
                    "~/.cargo/registry/src/**".to_owned(),
                    "~/.rustup/toolchains/**".to_owned(),
                ],
            },
        }
    }
}

/// Errors returned by [`Settings::load`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum SettingsError {
    /// Failed to read the settings file from disk.
    #[error("Failed to read settings file")]
    Read,
    /// Settings file could not be deserialized from YAML.
    #[error("Failed to deserialize settings YAML")]
    Deserialize,
}

/// Quote unquoted YAML list items starting with `!` so the YAML parser
/// treats them as strings rather than tags.
///
/// Transforms:
///
/// ```yaml
///  - !/foo/**
/// ````
///
/// Into:
///
/// ```yaml
///  - "!/foo/**"
/// ```
fn quote_yaml_tags(yaml: &str) -> String {
    let mut out = String::with_capacity(yaml.len());
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- !")
            && !rest.starts_with('"')
            && !rest.starts_with('\'')
        {
            let indent = &line[..line.len() - trimmed.len()];
            out.push_str(indent);
            out.push_str("- \"!");
            out.push_str(&rest.replace('"', "\\\""));
            out.push('"');
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Path to the settings file.
fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("config_dir should be valid")
        .join("claude-hooks")
        .join("settings.yaml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_negation_unquoted() {
        let yaml = "git:\n  paths:\n    - !/home/user/repos/forked/**\n";
        let settings: Settings =
            serde_yaml::from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["!/home/user/repos/forked/**"]);
    }

    #[test]
    fn yaml_negation_already_quoted() {
        let yaml = "git:\n  paths:\n    - \"!/home/user/repos/forked/**\"\n";
        let settings: Settings =
            serde_yaml::from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["!/home/user/repos/forked/**"]);
    }

    #[test]
    fn yaml_negation_single_quoted() {
        let yaml = "git:\n  paths:\n    - '!/home/user/repos/forked/**'\n";
        let settings: Settings =
            serde_yaml::from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["!/home/user/repos/forked/**"]);
    }

    #[test]
    fn yaml_non_negated_unchanged() {
        let yaml = "git:\n  paths:\n    - /home/user/repos/**\n";
        let settings: Settings =
            serde_yaml::from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["/home/user/repos/**"]);
    }

    #[test]
    fn yaml_mixed_patterns() {
        let yaml = "git:\n  paths:\n    - /home/user/repos/**\n    - !/home/user/repos/forked/**\n    - /home/user/repos/forked/this\n";
        let settings: Settings =
            serde_yaml::from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(
            settings.git.paths,
            vec![
                "/home/user/repos/**",
                "!/home/user/repos/forked/**",
                "/home/user/repos/forked/this",
            ]
        );
    }

    #[test]
    fn quote_yaml_tags_preserves_indentation() {
        assert_eq!(quote_yaml_tags("    - !/foo\n"), "    - \"!/foo\"\n");
    }

    #[test]
    fn quote_yaml_tags_escapes_inner_quotes() {
        assert_eq!(
            quote_yaml_tags("    - !/foo/\"bar\"\n"),
            "    - \"!/foo/\\\"bar\\\"\"\n",
        );
    }
}
