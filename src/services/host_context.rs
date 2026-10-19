//! System path context resolved at startup.

use crate::prelude::*;
use dirs::{config_dir, home_dir};

const APP_NAME: &str = "hook-rs";
const SETTINGS_FILE_NAME: &str = "settings.yaml";

/// System paths resolved at startup.
///
/// - Registered as an instance in the service container
/// - Provides home and config directories for tilde expansion and settings loading
pub struct HostContext {
    /// Home directory for tilde expansion.
    pub home_dir: PathBuf,
    /// Configuration file path for settings loading.
    pub config_file: PathBuf,
}

impl HostContext {
    /// Resolve system paths from the current environment.
    pub fn resolve() -> Self {
        Self {
            home_dir: home_dir().expect("home directory should be resolvable"),
            config_file: config_dir()
                .expect("config directory should be resolvable")
                .join(APP_NAME)
                .join(SETTINGS_FILE_NAME),
        }
    }

    /// Deterministic paths for testing.
    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            home_dir: PathBuf::from("/home/user"),
            config_file: PathBuf::from("/home/user/.config")
                .join(APP_NAME)
                .join(SETTINGS_FILE_NAME),
        }
    }
}

impl FromServices for HostContext {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::resolve())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_context_resolve() {
        let ctx = HostContext::resolve();
        assert!(ctx.home_dir.is_absolute());
        assert!(ctx.config_file.ends_with("hook-rs/settings.yaml"));
    }
}
