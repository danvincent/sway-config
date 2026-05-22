/// Test roundtrip serialization of settings model
/// Tests that settings can be serialized to TOML and deserialized back to the same state
use sway_configurator::model::settings::Settings;
use sway_configurator::model::theme::ThemeSelection;

#[test]
fn test_settings_default_serialization() {
    let settings = Settings::default();
    let toml_str = toml::to_string_pretty(&settings).expect("Failed to serialize settings");
    
    let deserialized: Settings = toml::from_str(&toml_str).expect("Failed to deserialize settings");
    
    assert_eq!(deserialized, settings);
}

#[test]
fn test_settings_with_theme_serialization() {
    let mut settings = Settings::default();
    settings.theme = Some(ThemeSelection {
        name: "dark-theme".to_string(),
        source: "system".to_string(),
        path: String::new(),
    });
    
    let toml_str = toml::to_string_pretty(&settings).expect("Failed to serialize settings");
    
    let deserialized: Settings = toml::from_str(&toml_str).expect("Failed to deserialize settings");
    
    assert_eq!(deserialized, settings);
    assert_eq!(deserialized.theme.as_ref().unwrap().name, "dark-theme");
}

#[test]
fn test_settings_idempotent_roundtrip() {
    let settings = Settings::default();
    
    let toml1 = toml::to_string_pretty(&settings).expect("Failed to serialize first time");
    let deserialized1: Settings = toml::from_str(&toml1).expect("Failed to deserialize first time");
    let toml2 = toml::to_string_pretty(&deserialized1).expect("Failed to serialize second time");
    
    assert_eq!(toml1, toml2, "Roundtrip serialization is not idempotent");
}
