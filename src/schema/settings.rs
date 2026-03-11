//! User-specific settings loaded from `~/.config/claude-hooks/settings.yaml`.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// User-specific settings for rule evaluation.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Settings {
    /// Git-specific settings for `git -C` path classification.
    #[serde(default)]
    pub git: GitSettings,
    /// GitHub bot username for auto-approving PR comments.
    #[serde(default)]
    pub bot_username: String,
    /// GitHub org prefix for auto-approving bot operations.
    #[serde(default)]
    pub bot_org: String,
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
    /// Load settings from `~/.config/claude-hooks/settings.yaml`.
    ///
    /// Returns `Settings::default()` on any failure (missing file, parse error, etc.).
    #[must_use]
    pub fn load() -> Self {
        Self::path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| serde_yaml::from_str(&contents).ok())
            .unwrap_or_default()
    }

    /// Path to the settings file.
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("claude-hooks").join("settings.yaml"))
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
            bot_username: "StudioLE-Bot".to_owned(),
            bot_org: "StudioLE".to_owned(),
        }
    }
}
