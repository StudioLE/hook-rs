//! Root settings struct and file loading.

use crate::prelude::*;
use std::fs::read_to_string;

/// User-specific settings for rule evaluation.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Settings {
    /// Git-specific settings for `git -C` path classification.
    #[serde(default)]
    pub git: GitSettings,
    /// Read tool settings for auto-allowing trusted file paths.
    #[serde(default)]
    pub read: ReadSettings,
    /// Worktree settings for `git worktree add` path classification.
    #[serde(default)]
    pub worktrees: WorktreeSettings,
}

impl Settings {
    /// Load settings from file.
    pub fn from_file(path: &Path) -> Result<Self, Report<SettingsError>> {
        if !path.exists() {
            return Err(Report::new(SettingsError::NotFound).attach_path(path));
        }
        let raw = read_to_string(path)
            .change_context(SettingsError::Read)
            .attach_path(path)?;
        let yaml = quote_yaml_tags(&raw);
        let settings: Settings = yaml_from_str(&yaml)
            .change_context(SettingsError::Deserialize)
            .attach_path(path)?;
        trace!(
            path = %path.display(),
            git_paths = settings.git.paths.len(),
            read_paths = settings.read.paths.len(),
            worktrees_paths = settings.worktrees.paths.len(),
            "Loaded settings",
        );
        Ok(settings)
    }

    /// Create [`Settings`] with only [`GitSettings`].
    #[cfg(test)]
    #[must_use]
    pub fn with_git(paths: &[&str]) -> Self {
        Self {
            git: GitSettings::new(paths),
            ..Default::default()
        }
    }

    /// Create [`Settings`] with only [`ReadSettings`].
    #[cfg(test)]
    #[must_use]
    pub fn with_read(paths: &[&str]) -> Self {
        Self {
            read: ReadSettings::new(paths),
            ..Default::default()
        }
    }

    /// Create [`Settings`] with only [`WorktreeSettings`].
    #[cfg(test)]
    #[must_use]
    pub fn with_worktrees(paths: &[&str]) -> Self {
        Self {
            worktrees: WorktreeSettings::new(paths),
            ..Default::default()
        }
    }

    /// Mock settings for use in tests.
    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            git: GitSettings::new(&["/home/user/repos/**", "!/home/user/repos/forked/**"]),
            read: ReadSettings::new(&[
                "~/.cargo/registry/src/**",
                "~/.rustup/toolchains/**",
                "/path/to/repos/**",
                "README.md",
                "!.env",
                "!.env.*",
            ]),
            worktrees: WorktreeSettings::new(&["/home/user/worktrees/**"]),
        }
    }
}

impl FromServices for Settings {
    type Error = SettingsError;

    fn from_services(services: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        let host = services
            .get::<HostContext>()
            .change_context(SettingsError::Resolve)?;
        Self::from_file(&host.config_file)
    }
}

/// Errors returned by [`Settings`] construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum SettingsError {
    /// Failed to resolve host context.
    #[error("Failed to resolve host context")]
    Resolve,
    /// Settings file not found at the expected path.
    #[error("Failed to find settings file")]
    NotFound,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_negation_unquoted() {
        let yaml = "git:\n  paths:\n    - !/home/user/repos/forked/**\n";
        let settings: Settings = yaml_from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["!/home/user/repos/forked/**"]);
    }

    #[test]
    fn yaml_negation_already_quoted() {
        let yaml = "git:\n  paths:\n    - \"!/home/user/repos/forked/**\"\n";
        let settings: Settings = yaml_from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["!/home/user/repos/forked/**"]);
    }

    #[test]
    fn yaml_negation_single_quoted() {
        let yaml = "git:\n  paths:\n    - '!/home/user/repos/forked/**'\n";
        let settings: Settings = yaml_from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["!/home/user/repos/forked/**"]);
    }

    #[test]
    fn yaml_non_negated_unchanged() {
        let yaml = "git:\n  paths:\n    - /home/user/repos/**\n";
        let settings: Settings = yaml_from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(settings.git.paths, vec!["/home/user/repos/**"]);
    }

    #[test]
    fn yaml_mixed_patterns() {
        let yaml = "git:\n  paths:\n    - /home/user/repos/**\n    - !/home/user/repos/forked/**\n    - /home/user/repos/forked/this\n";
        let settings: Settings = yaml_from_str(&quote_yaml_tags(yaml)).expect("should parse");
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
    fn yaml_worktree_paths() {
        let yaml = "worktrees:\n  paths:\n    - /home/user/worktrees/**\n    - !/home/user/worktrees/blocked/**\n";
        let settings: Settings = yaml_from_str(&quote_yaml_tags(yaml)).expect("should parse");
        assert_eq!(
            settings.worktrees.paths,
            vec![
                "/home/user/worktrees/**",
                "!/home/user/worktrees/blocked/**"
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
