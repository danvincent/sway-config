/// Settings model - editable user configuration
use serde::{Deserialize, Serialize};
use crate::model::theme::ThemeSelection;
use crate::model::output::OutputConfig;
use crate::model::input::{KeyboardConfig, TouchpadConfig};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Currently selected theme
    pub theme: Option<ThemeSelection>,
    /// Output/display configurations
    pub outputs: Vec<OutputConfig>,
    /// Keyboard configurations
    pub keyboards: Vec<KeyboardConfig>,
    /// Touchpad configurations
    pub touchpads: Vec<TouchpadConfig>,
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
}
