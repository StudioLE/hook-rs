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
/// Read-only `git -C <path>` commands are auto-allowed when the path is under
/// a trusted directory and not under an untrusted subdirectory.
///
/// See CVE-2025-59536 and CVE-2026-21852.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GitSettings {
    /// Directories where read-only `git -C` operations are auto-allowed.
    #[serde(default)]
    pub trusted_dirs: Vec<String>,
    /// Subdirectories of trusted dirs that should be excluded (e.g. forks).
    #[serde(default)]
    pub untrusted_dirs: Vec<String>,
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
        let yaml = fs::read_to_string(&path).change_context(SettingsError::Read)?;
        let settings: Settings =
            serde_yaml::from_str(&yaml).change_context(SettingsError::Deserialize)?;
        trace!(
            path = %path.display(),
            git_trusted = settings.git.trusted_dirs.len(),
            git_untrusted = settings.git.untrusted_dirs.len(),
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
                trusted_dirs: vec!["/home/user/repos/".to_owned()],
                untrusted_dirs: vec!["/home/user/repos/forked/".to_owned()],
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

/// Path to the settings file.
fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("config_dir should be valid")
        .join("claude-hooks")
        .join("settings.yaml")
}
