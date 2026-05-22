use crate::model::autostart::AutostartConfig;
use crate::model::general::GeneralConfig;
use crate::model::idle::IdleConfig;
use crate::model::input::{KeyboardConfig, TouchpadConfig};
use crate::model::notifications::NotificationsConfig;
use crate::model::output::OutputConfig;
/// Pure read helpers — extract model config from Settings without any GTK.
/// These are headless-testable and used by UI pages to populate widgets.
use crate::model::settings::Settings;
use crate::model::theme::ThemeSelection;
use crate::model::waybar::WaybarConfig;

pub fn read_idle(settings: &Settings) -> IdleConfig {
    settings.idle.clone()
}

pub fn read_waybar(settings: &Settings) -> WaybarConfig {
    settings.waybar.clone()
}

pub fn read_notifications(settings: &Settings) -> NotificationsConfig {
    settings.notifications.clone()
}

pub fn read_outputs(settings: &Settings) -> Vec<OutputConfig> {
    settings.outputs.clone()
}

pub fn read_keyboards(settings: &Settings) -> Vec<KeyboardConfig> {
    settings.keyboards.clone()
}

pub fn read_touchpads(settings: &Settings) -> Vec<TouchpadConfig> {
    settings.touchpads.clone()
}

pub fn read_autostart(settings: &Settings) -> AutostartConfig {
    settings.autostart.clone()
}

pub fn read_theme(settings: &Settings) -> Option<ThemeSelection> {
    settings.theme.clone()
}

pub fn read_general(settings: &Settings) -> GeneralConfig {
    settings.general.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::autostart::{AutostartConfig, AutostartEntry};
    use crate::model::idle::IdleConfig;
    use crate::model::notifications::NotificationsConfig;
    use crate::model::settings::Settings;
    use crate::model::theme::ThemeSelection;
    use crate::model::waybar::WaybarConfig;

    #[test]
    fn test_read_idle_roundtrip() {
        let mut settings = Settings::default();
        settings.idle.lock_timeout = 600;
        settings.idle.lock_command = "waylock".to_string();
        let config = read_idle(&settings);
        assert_eq!(config.lock_timeout, 600);
        assert_eq!(config.lock_command, "waylock");
    }

    #[test]
    fn test_read_waybar_roundtrip() {
        let mut settings = Settings::default();
        settings.waybar.height = 42;
        settings.waybar.enabled = false;
        let config = read_waybar(&settings);
        assert_eq!(config.height, 42);
        assert!(!config.enabled);
    }

    #[test]
    fn test_read_notifications_roundtrip() {
        let mut settings = Settings::default();
        settings.notifications.timeout_ms = 8000;
        settings.notifications.max_visible = 3;
        let config = read_notifications(&settings);
        assert_eq!(config.timeout_ms, 8000);
        assert_eq!(config.max_visible, 3);
    }

    #[test]
    fn test_read_outputs_roundtrip() {
        let settings = Settings::default();
        let outputs = read_outputs(&settings);
        assert!(outputs.is_empty());
    }

    #[test]
    fn test_read_keyboards_roundtrip() {
        let settings = Settings::default();
        let keyboards = read_keyboards(&settings);
        assert!(keyboards.is_empty());
    }

    #[test]
    fn test_read_touchpads_roundtrip() {
        let settings = Settings::default();
        let touchpads = read_touchpads(&settings);
        assert!(touchpads.is_empty());
    }

    #[test]
    fn test_read_autostart_roundtrip() {
        let mut settings = Settings::default();
        settings.autostart.entries.push(AutostartEntry {
            id: "test".to_string(),
            command: "nm-applet".to_string(),
            enabled: true,
            description: "Network Manager".to_string(),
        });
        let config = read_autostart(&settings);
        assert_eq!(config.entries.len(), 1);
        assert_eq!(config.entries[0].command, "nm-applet");
    }

    #[test]
    fn test_read_theme_roundtrip() {
        let mut settings = Settings::default();
        settings.theme = Some(ThemeSelection::new("my-theme", "local"));
        let theme = read_theme(&settings);
        assert_eq!(theme.unwrap().name, "my-theme");
    }

    #[test]
    fn test_read_theme_returns_none_for_default() {
        let settings = Settings::default();
        assert!(read_theme(&settings).is_none());
    }

    #[test]
    fn test_read_general_roundtrip() {
        let mut settings = Settings::default();
        settings.general.terminal = "kitty".to_string();
        let config = read_general(&settings);
        assert_eq!(config.terminal, "kitty");
    }
}
