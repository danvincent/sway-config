/// Integration tests for settings with outputs, keyboards, and touchpads
use sway_configurator::model::settings::Settings;
use sway_configurator::model::output::{OutputConfig, Resolution, Position, Transform};
use sway_configurator::model::input::{KeyboardConfig, TouchpadConfig, AccelProfile};

#[test]
fn test_settings_with_outputs_roundtrip() {
    let mut settings = Settings::default();
    
    settings.outputs.push(OutputConfig {
        name: "eDP-1".to_string(),
        enabled: true,
        resolution: Some(Resolution {
            width: 1920,
            height: 1080,
        }),
        refresh_rate: Some(60000),
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&settings)
        .expect("Failed to serialize settings to TOML");
    
    // Deserialize from TOML
    let loaded_settings: Settings = toml::from_str(&toml_str)
        .expect("Failed to deserialize settings from TOML");
    
    // Check round-trip
    assert_eq!(settings, loaded_settings);
    assert_eq!(loaded_settings.outputs.len(), 1);
    assert_eq!(loaded_settings.outputs[0].name, "eDP-1");
    assert!(loaded_settings.outputs[0].enabled);
}

#[test]
fn test_settings_with_keyboards_roundtrip() {
    let mut settings = Settings::default();
    
    settings.keyboards.push(KeyboardConfig {
        identifier: "1:1:test_keyboard".to_string(),
        xkb_layout: "us".to_string(),
        xkb_variant: String::new(),
        xkb_options: String::new(),
        repeat_delay: 600,
        repeat_rate: 25,
    });
    
    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&settings)
        .expect("Failed to serialize settings to TOML");
    
    // Deserialize from TOML
    let loaded_settings: Settings = toml::from_str(&toml_str)
        .expect("Failed to deserialize settings from TOML");
    
    // Check round-trip
    assert_eq!(settings, loaded_settings);
    assert_eq!(loaded_settings.keyboards.len(), 1);
    assert_eq!(loaded_settings.keyboards[0].identifier, "1:1:test_keyboard");
}

#[test]
fn test_settings_with_touchpads_roundtrip() {
    let mut settings = Settings::default();
    
    settings.touchpads.push(TouchpadConfig {
        identifier: "2:7:test_touchpad".to_string(),
        tap_to_click: true,
        natural_scroll: false,
        dwt: true,
        accel_speed: 0.5,
        accel_profile: AccelProfile::Adaptive,
        left_handed: false,
        middle_emulation: false,
    });
    
    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&settings)
        .expect("Failed to serialize settings to TOML");
    
    // Deserialize from TOML
    let loaded_settings: Settings = toml::from_str(&toml_str)
        .expect("Failed to deserialize settings from TOML");
    
    // Check round-trip
    assert_eq!(settings, loaded_settings);
    assert_eq!(loaded_settings.touchpads.len(), 1);
    assert_eq!(loaded_settings.touchpads[0].identifier, "2:7:test_touchpad");
    assert!(loaded_settings.touchpads[0].tap_to_click);
}

#[test]
fn test_settings_backward_compat() {
    // Old TOML without new fields should still load
    let old_toml = r#"
[theme]
name = "dark"
source = "system"
"#;
    
    let settings: Settings = toml::from_str(old_toml)
        .expect("Failed to deserialize old TOML format");
    
    // Should have defaults for new fields
    assert_eq!(settings.outputs.len(), 0);
    assert_eq!(settings.keyboards.len(), 0);
    assert_eq!(settings.touchpads.len(), 0);
    assert!(settings.theme.is_some());
}

#[test]
fn test_settings_all_fields_roundtrip() {
    let mut settings = Settings::default();
    
    settings.outputs.push(OutputConfig {
        name: "eDP-1".to_string(),
        enabled: true,
        resolution: Some(Resolution {
            width: 1920,
            height: 1080,
        }),
        refresh_rate: Some(60000),
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    settings.keyboards.push(KeyboardConfig {
        identifier: "1:1:test_keyboard".to_string(),
        xkb_layout: "us".to_string(),
        xkb_variant: String::new(),
        xkb_options: String::new(),
        repeat_delay: 600,
        repeat_rate: 25,
    });
    
    settings.touchpads.push(TouchpadConfig {
        identifier: "2:7:test_touchpad".to_string(),
        tap_to_click: true,
        natural_scroll: false,
        dwt: true,
        accel_speed: 0.5,
        accel_profile: AccelProfile::Adaptive,
        left_handed: false,
        middle_emulation: false,
    });
    
    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&settings)
        .expect("Failed to serialize settings to TOML");
    
    // Deserialize from TOML
    let loaded_settings: Settings = toml::from_str(&toml_str)
        .expect("Failed to deserialize settings from TOML");
    
    // Check round-trip
    assert_eq!(settings, loaded_settings);
    assert_eq!(loaded_settings.outputs.len(), 1);
    assert_eq!(loaded_settings.keyboards.len(), 1);
    assert_eq!(loaded_settings.touchpads.len(), 1);
}
