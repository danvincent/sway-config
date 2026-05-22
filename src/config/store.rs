/// Settings store - loads and saves configuration to disk
use crate::model::settings::Settings;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SettingsStore {
    /// Path where settings are persisted
    pub path: PathBuf,
    /// Currently loaded settings
    pub settings: Settings,
}

impl SettingsStore {
    /// Create a new store with the given path
    pub fn new(path: impl AsRef<Path>) -> Self {
        SettingsStore {
            path: path.as_ref().to_path_buf(),
            settings: Settings::default(),
        }
    }

    /// Pure function to resolve config path given optional XDG_CONFIG_HOME and HOME env vars
    /// This function doesn't access the process environment, making it safe to test without mutation
    ///
    /// # Arguments
    /// * `xdg_config_home` - Optional value of XDG_CONFIG_HOME environment variable
    /// * `home` - Optional value of HOME environment variable
    ///
    /// # Returns
    /// Path to `sway-config/settings.toml` using XDG conventions
    pub fn resolve_config_path(xdg_config_home: Option<&str>, home: Option<&str>) -> PathBuf {
        let config_dir = xdg_config_home
            .and_then(|xdg| {
                if xdg.is_empty() {
                    None
                } else {
                    Some(xdg.to_string())
                }
            })
            .or_else(|| home.map(|home| format!("{}/.config", home)))
            .unwrap_or_else(|| ".config".to_string());

        PathBuf::from(config_dir)
            .join("sway-config")
            .join("settings.toml")
    }

    /// Resolve the default XDG config path for sway-config settings
    /// Returns: `$XDG_CONFIG_HOME/sway-config/settings.toml` (or `~/.config/sway-config/settings.toml` if XDG_CONFIG_HOME is unset)
    pub fn default_path() -> PathBuf {
        Self::resolve_config_path(
            env::var("XDG_CONFIG_HOME").ok().as_deref(),
            env::var("HOME").ok().as_deref(),
        )
    }

    /// Load settings from the default XDG config location
    pub fn open_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load(Self::default_path())
    }

    /// Load settings from disk, or create defaults if file doesn't exist
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref().to_path_buf();

        let settings = if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content)?
        } else {
            Settings::default()
        };

        Ok(SettingsStore { path, settings })
    }

    /// Save current settings to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&self.settings)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    /// Load settings from a base directory (base_dir/settings.toml)
    pub fn open_at(base_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = base_dir.join("settings.toml");
        Self::load(path)
    }

    /// Create a test store using a temporary directory; settings start as defaults.
    /// Intended for use with SWAY_CONFIG_TEST=1.
    pub fn test_store(dir: &Path) -> Self {
        let path = dir.join("settings.toml");
        SettingsStore {
            path,
            settings: Settings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_new() {
        let store = SettingsStore::new("/tmp/test.toml");
        assert!(store.settings.theme.is_none());
    }

    #[test]
    fn test_default_path_ends_with_settings_toml() {
        let path = SettingsStore::default_path();
        assert!(path
            .to_string_lossy()
            .ends_with("sway-config/settings.toml"));
    }

    #[test]
    fn test_default_path_is_absolute() {
        let path = SettingsStore::default_path();
        assert!(
            path.is_absolute(),
            "default_path should return an absolute path"
        );
    }

    #[test]
    fn test_default_path_includes_sway_config() {
        let path = SettingsStore::default_path();
        assert!(
            path.to_string_lossy().contains("sway-config"),
            "default_path should include 'sway-config' directory"
        );
    }

    // Tests for resolve_config_path - pure function, no env mutation needed

    #[test]
    fn test_resolve_config_path_uses_xdg_config_home_when_set() {
        let path = SettingsStore::resolve_config_path(Some("/custom/config"), Some("/home/user"));
        assert!(
            path.to_string_lossy()
                .starts_with("/custom/config/sway-config"),
            "resolve_config_path should use XDG_CONFIG_HOME when set: {}",
            path.display()
        );
    }

    #[test]
    fn test_resolve_config_path_ignores_empty_xdg_config_home() {
        let path = SettingsStore::resolve_config_path(Some(""), Some("/home/user"));
        assert!(
            path.to_string_lossy()
                .contains("/home/user/.config/sway-config"),
            "resolve_config_path should ignore empty XDG_CONFIG_HOME: {}",
            path.display()
        );
    }

    #[test]
    fn test_resolve_config_path_falls_back_to_home_config_when_xdg_unset() {
        let path = SettingsStore::resolve_config_path(None, Some("/home/testuser"));
        assert!(
            path.to_string_lossy().contains("/home/testuser/.config/sway-config"),
            "resolve_config_path should fall back to $HOME/.config when XDG_CONFIG_HOME is unset: {}",
            path.display()
        );
    }

    #[test]
    fn test_resolve_config_path_uses_fallback_when_both_unset() {
        let path = SettingsStore::resolve_config_path(None, None);
        assert!(
            path.to_string_lossy().contains(".config/sway-config"),
            "resolve_config_path should use .config fallback when both env vars are unset: {}",
            path.display()
        );
    }

    #[test]
    fn test_open_at_creates_store_with_correct_path() {
        let dir = std::env::temp_dir().join("sway-config-test-open-at");
        std::fs::create_dir_all(&dir).unwrap();
        let store = SettingsStore::open_at(&dir).unwrap();
        assert!(store.path.ends_with("settings.toml"));
        assert!(store.path.starts_with(&dir));
    }

    #[test]
    fn test_test_store_uses_provided_dir() {
        let dir = std::env::temp_dir().join("sway-config-test-store");
        let store = SettingsStore::test_store(&dir);
        assert!(store.path.ends_with("settings.toml"));
        assert!(store.path.starts_with(&dir));
        assert!(store.settings.theme.is_none());
    }

    #[test]
    fn test_open_at_roundtrip_save_and_load() {
        let dir = std::env::temp_dir().join("sway-config-test-roundtrip");
        std::fs::create_dir_all(&dir).unwrap();
        let mut store = SettingsStore::open_at(&dir).unwrap();
        store.settings.idle.lock_timeout = 999;
        store.save().unwrap();

        let reloaded = SettingsStore::open_at(&dir).unwrap();
        assert_eq!(reloaded.settings.idle.lock_timeout, 999);
    }
}
