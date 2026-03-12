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
    pub trusted_dirs: Vec<String>,
    /// Subdirectories of trusted dirs that should be excluded (e.g. forks).
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
            read: ReadSettings {
                paths: vec![
                    "~/.cargo/registry/src/**".to_owned(),
                    "~/.rustup/toolchains/**".to_owned(),
                ],
            },
        }
    }
}
