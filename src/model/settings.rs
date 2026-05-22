/// Settings model - editable user configuration
use serde::{Deserialize, Serialize};
use crate::model::theme::{ThemeSelection, ThemeOverrides};
use crate::model::output::OutputConfig;
use crate::model::input::{KeyboardConfig, TouchpadConfig};
use crate::model::idle::IdleConfig;
use crate::model::waybar::WaybarConfig;
use crate::model::autostart::AutostartConfig;
use crate::model::notifications::NotificationsConfig;
use crate::model::general::GeneralConfig;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Currently selected theme
    pub theme: Option<ThemeSelection>,
    /// Overrides applied on top of the selected theme's .env values
    #[serde(default)]
    pub theme_overrides: ThemeOverrides,
    /// Output/display configurations
    pub outputs: Vec<OutputConfig>,
    /// Keyboard configurations
    pub keyboards: Vec<KeyboardConfig>,
    /// Touchpad configurations
    pub touchpads: Vec<TouchpadConfig>,
    /// Idle behavior configuration
    #[serde(default)]
    pub idle: IdleConfig,
    /// Waybar configuration
    #[serde(default)]
    pub waybar: WaybarConfig,
    /// Autostart entries
    #[serde(default)]
    pub autostart: AutostartConfig,
    /// Notifications configuration
    #[serde(default)]
    pub notifications: NotificationsConfig,
    /// General sway configuration (terminal, etc.)
    #[serde(default)]
    pub general: GeneralConfig,
    /// Optional custom path to scan for additional themes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_themes_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(settings.theme.is_none());
    }

    #[test]
    fn test_settings_with_theme() {
        let mut settings = Settings::default();
        settings.theme = Some(ThemeSelection::new("dark", "system"));
        assert_eq!(settings.theme.as_ref().unwrap().name, "dark");
    }

    #[test]
    fn test_settings_custom_themes_path_defaults_to_none() {
        let settings = Settings::default();
        assert!(settings.custom_themes_path.is_none());
    }

    #[test]
    fn test_settings_custom_themes_path_roundtrip() {
        let mut settings = Settings::default();
        settings.custom_themes_path = Some("/home/user/my-themes".to_string());
        
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let reloaded: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(reloaded.custom_themes_path, Some("/home/user/my-themes".to_string()));
    }

    #[test]
    fn test_settings_custom_themes_path_none_not_serialized() {
        let settings = Settings::default(); // custom_themes_path = None
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        assert!(!toml_str.contains("custom_themes_path"),
            "None custom_themes_path should not appear in serialized TOML");
    }

    #[test]
    fn test_settings_default_terminal_is_empty() {
        let settings = Settings::default();
        assert!(settings.general.terminal.is_empty());
    }

    #[test]
    fn test_settings_general_terminal_roundtrip() {
        let mut settings = Settings::default();
        settings.general.terminal = "foot".to_string();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let reloaded: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(reloaded.general.terminal, "foot");
    }
}
