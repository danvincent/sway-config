/// Tests for Phase 4 feature-detected workstation settings
use sway_configurator::model::idle::IdleConfig;
use sway_configurator::model::waybar::{WaybarConfig, WaybarModule, BarPosition};
use sway_configurator::model::autostart::{AutostartConfig, AutostartEntry};
use sway_configurator::model::notifications::{NotificationsConfig, NotifPosition};
use sway_configurator::model::settings::Settings;

// ============================================================================
// Idle Model Tests
// ============================================================================

#[test]
fn test_idle_config_default() {
    let idle = IdleConfig::default();
    assert_eq!(idle.lock_timeout, 300);
    assert_eq!(idle.screen_off_timeout, 0);
    assert_eq!(idle.lock_command, "swaylock");
    assert!(idle.before_sleep);
}

#[test]
fn test_idle_config_roundtrip() {
    let mut idle = IdleConfig::default();
    idle.lock_timeout = 600;
    idle.screen_off_timeout = 900;
    idle.lock_command = "swaylock-effects".to_string();
    idle.before_sleep = false;

    let toml_str = toml::to_string_pretty(&idle).expect("Failed to serialize idle config");
    let deserialized: IdleConfig = toml::from_str(&toml_str).expect("Failed to deserialize idle config");

    assert_eq!(deserialized, idle);
    assert_eq!(deserialized.lock_timeout, 600);
    assert_eq!(deserialized.screen_off_timeout, 900);
    assert_eq!(deserialized.lock_command, "swaylock-effects");
    assert!(!deserialized.before_sleep);
}

#[test]
fn test_idle_config_zero_disables() {
    let mut idle = IdleConfig::default();
    idle.lock_timeout = 0;
    idle.screen_off_timeout = 0;

    let toml_str = toml::to_string_pretty(&idle).expect("Failed to serialize idle config");
    let deserialized: IdleConfig = toml::from_str(&toml_str).expect("Failed to deserialize idle config");

    assert_eq!(deserialized.lock_timeout, 0);
    assert_eq!(deserialized.screen_off_timeout, 0);
}

// ============================================================================
// Waybar Model Tests
// ============================================================================

#[test]
fn test_waybar_config_default() {
    let waybar = WaybarConfig::default();
    assert!(waybar.enabled);
    assert_eq!(waybar.position, BarPosition::Top);
    assert_eq!(waybar.height, 30);
    assert!(waybar.tray);
}

#[test]
fn test_waybar_config_roundtrip() {
    let mut waybar = WaybarConfig::default();
    waybar.enabled = false;
    waybar.position = BarPosition::Bottom;
    waybar.height = 40;
    waybar.tray = false;

    let toml_str = toml::to_string_pretty(&waybar).expect("Failed to serialize waybar config");
    let deserialized: WaybarConfig = toml::from_str(&toml_str).expect("Failed to deserialize waybar config");

    assert_eq!(deserialized, waybar);
    assert!(!deserialized.enabled);
    assert_eq!(deserialized.position, BarPosition::Bottom);
    assert_eq!(deserialized.height, 40);
    assert!(!deserialized.tray);
}

#[test]
fn test_waybar_module_new() {
    let module = WaybarModule::new("battery", true);
    assert_eq!(module.name, "battery");
    assert!(module.enabled);

    let module2 = WaybarModule::new("network", false);
    assert_eq!(module2.name, "network");
    assert!(!module2.enabled);
}

#[test]
fn test_bar_position_default() {
    let pos = BarPosition::default();
    assert_eq!(pos, BarPosition::Top);
}

#[test]
fn test_waybar_modules_roundtrip() {
    let mut waybar = WaybarConfig::default();
    waybar.modules_right.push(WaybarModule::new("battery", true));
    waybar.modules_right.push(WaybarModule::new("network", false));

    let toml_str = toml::to_string_pretty(&waybar).expect("Failed to serialize waybar config");
    let deserialized: WaybarConfig = toml::from_str(&toml_str).expect("Failed to deserialize waybar config");

    assert_eq!(deserialized.modules_right.len(), 2);
    assert_eq!(deserialized.modules_right[0].name, "battery");
    assert!(deserialized.modules_right[0].enabled);
    assert_eq!(deserialized.modules_right[1].name, "network");
    assert!(!deserialized.modules_right[1].enabled);
}

// ============================================================================
// Autostart Model Tests
// ============================================================================

#[test]
fn test_autostart_add_entry() {
    let mut config = AutostartConfig::new();
    let entry = AutostartEntry {
        id: "test-app".to_string(),
        command: "test-app --flag".to_string(),
        enabled: true,
        description: "Test Application".to_string(),
    };

    config.add(entry.clone());

    assert_eq!(config.entries.len(), 1);
    assert_eq!(config.entries[0].id, "test-app");
}

#[test]
fn test_autostart_remove_entry() {
    let mut config = AutostartConfig::new();
    let entry = AutostartEntry {
        id: "test-app".to_string(),
        command: "test-app --flag".to_string(),
        enabled: true,
        description: "Test Application".to_string(),
    };

    config.add(entry);
    assert_eq!(config.entries.len(), 1);

    config.remove("test-app");
    assert_eq!(config.entries.len(), 0);
}

#[test]
fn test_autostart_remove_nonexistent() {
    let mut config = AutostartConfig::new();
    config.remove("nonexistent");
    assert_eq!(config.entries.len(), 0);
}

#[test]
fn test_autostart_find_entry() {
    let mut config = AutostartConfig::new();

    let entry1 = AutostartEntry {
        id: "app1".to_string(),
        command: "app1".to_string(),
        enabled: true,
        description: "App 1".to_string(),
    };
    let entry2 = AutostartEntry {
        id: "app2".to_string(),
        command: "app2".to_string(),
        enabled: false,
        description: "App 2".to_string(),
    };

    config.add(entry1);
    config.add(entry2);

    let found = config.find("app1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().command, "app1");

    let found2 = config.find("app2");
    assert!(found2.is_some());
    assert_eq!(found2.unwrap().enabled, false);

    let not_found = config.find("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_autostart_config_roundtrip() {
    let mut config = AutostartConfig::new();

    config.add(AutostartEntry {
        id: "app1".to_string(),
        command: "app1 --flag".to_string(),
        enabled: true,
        description: "Application 1".to_string(),
    });

    config.add(AutostartEntry {
        id: "app2".to_string(),
        command: "app2".to_string(),
        enabled: false,
        description: "Application 2".to_string(),
    });

    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize autostart config");
    let deserialized: AutostartConfig = toml::from_str(&toml_str).expect("Failed to deserialize autostart config");

    assert_eq!(deserialized.entries.len(), 2);
    assert_eq!(deserialized.entries[0].id, "app1");
    assert!(deserialized.entries[0].enabled);
    assert_eq!(deserialized.entries[1].id, "app2");
    assert!(!deserialized.entries[1].enabled);
}

// ============================================================================
// Notifications Model Tests
// ============================================================================

#[test]
fn test_notifications_config_default() {
    let notif = NotificationsConfig::default();
    assert_eq!(notif.timeout_ms, 5000);
    assert_eq!(notif.max_visible, 5);
    assert_eq!(notif.position, NotifPosition::TopRight);
    assert!(!notif.follow_focus);
}

#[test]
fn test_notifications_config_roundtrip() {
    let mut notif = NotificationsConfig::default();
    notif.timeout_ms = 3000;
    notif.max_visible = 10;
    notif.position = NotifPosition::BottomLeft;
    notif.follow_focus = true;

    let toml_str = toml::to_string_pretty(&notif).expect("Failed to serialize notifications config");
    let deserialized: NotificationsConfig = toml::from_str(&toml_str).expect("Failed to deserialize notifications config");

    assert_eq!(deserialized, notif);
    assert_eq!(deserialized.timeout_ms, 3000);
    assert_eq!(deserialized.max_visible, 10);
    assert_eq!(deserialized.position, NotifPosition::BottomLeft);
    assert!(deserialized.follow_focus);
}

#[test]
fn test_notif_position_default() {
    let pos = NotifPosition::default();
    assert_eq!(pos, NotifPosition::TopRight);
}

// ============================================================================
// Settings Integration Tests
// ============================================================================

#[test]
fn test_settings_with_all_new_fields_roundtrip() {
    let mut settings = Settings::default();

    // Add idle config
    settings.idle.lock_timeout = 600;
    settings.idle.before_sleep = false;

    // Add waybar config
    settings.waybar.enabled = false;
    settings.waybar.height = 40;
    settings.waybar.modules_right.push(WaybarModule::new("battery", true));

    // Add autostart entries
    settings.autostart.add(AutostartEntry {
        id: "test".to_string(),
        command: "test-cmd".to_string(),
        enabled: true,
        description: "Test".to_string(),
    });

    // Add notifications config
    settings.notifications.timeout_ms = 3000;
    settings.notifications.max_visible = 8;

    // Roundtrip
    let toml_str = toml::to_string_pretty(&settings).expect("Failed to serialize settings");
    let deserialized: Settings = toml::from_str(&toml_str).expect("Failed to deserialize settings");

    assert_eq!(deserialized.idle.lock_timeout, 600);
    assert!(!deserialized.idle.before_sleep);
    assert!(!deserialized.waybar.enabled);
    assert_eq!(deserialized.waybar.height, 40);
    assert_eq!(deserialized.waybar.modules_right.len(), 1);
    assert_eq!(deserialized.autostart.entries.len(), 1);
    assert_eq!(deserialized.notifications.timeout_ms, 3000);
    assert_eq!(deserialized.notifications.max_visible, 8);
}

#[test]
fn test_settings_all_fields_backward_compat() {
    // Load a TOML with none of the new fields
    let old_toml = r#"
[theme]
name = "dark"
source = "system"
"#;

    let settings: Settings = toml::from_str(old_toml)
        .expect("Failed to deserialize old TOML format");

    // Should have defaults for new fields
    assert_eq!(settings.idle.lock_timeout, 300);
    assert!(settings.idle.before_sleep);
    assert_eq!(settings.idle.lock_command, "swaylock");

    assert!(settings.waybar.enabled);
    assert_eq!(settings.waybar.position, BarPosition::Top);
    assert_eq!(settings.waybar.height, 30);

    assert_eq!(settings.autostart.entries.len(), 0);

    assert_eq!(settings.notifications.timeout_ms, 5000);
    assert_eq!(settings.notifications.max_visible, 5);
    assert_eq!(settings.notifications.position, NotifPosition::TopRight);
}

// ============================================================================
// Feature Detection Smoke Tests
// ============================================================================

#[test]
fn test_has_swayidle_returns_bool() {
    // Test that the function returns a bool without panicking
    // We can't assert the value since it depends on whether swayidle is installed
    let result = sway_configurator::config::feature::has_swayidle();
    let _ = result; // suppress unused warning, we just need it to not panic
}

#[test]
fn test_has_waybar_returns_bool() {
    // Test that the function returns a bool without panicking
    let result = sway_configurator::config::feature::has_waybar();
    let _ = result;
}

#[test]
fn test_has_battery_returns_bool() {
    // Test that the function returns a bool without panicking
    let result = sway_configurator::config::feature::has_battery();
    let _ = result;
}

#[test]
fn test_has_screen_locker_returns_bool() {
    // Test that the function returns a bool without panicking
    let result = sway_configurator::config::feature::has_screen_locker();
    let _ = result;
}

#[test]
fn test_has_audio_returns_bool() {
    // Test that the function returns a bool without panicking
    let result = sway_configurator::config::feature::has_audio();
    let _ = result;
}

#[test]
fn test_has_network_manager_returns_bool() {
    // Test that the function returns a bool without panicking
    let result = sway_configurator::config::feature::has_network_manager();
    let _ = result;
}

#[test]
fn test_has_notification_daemon_returns_bool() {
    // Test that the function returns a bool without panicking
    let result = sway_configurator::config::feature::has_notification_daemon();
    let _ = result;
}

// Test with known-present binary (like "ls" or "sh")
#[test]
fn test_feature_detection_with_known_binary() {
    // These should return true because 'sh' is virtually always available
    use std::process::Command;
    
    // Check if we can actually use 'which' with 'sh'
    let output = Command::new("which")
        .arg("sh")
        .output();
    
    if output.is_ok() && output.unwrap().status.success() {
        // If 'which sh' works, our implementation should also work
        // This is just a smoke test to ensure no panics occur
        let _ = sway_configurator::config::feature::has_swayidle();
    }
}
