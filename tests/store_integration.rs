/// Test configuration store load/save
/// Tests persistence of settings to disk and loading them back
use sway_configurator::config::store::SettingsStore;
use tempfile::TempDir;

#[test]
fn test_store_save_and_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let store_path = temp_dir.path().join("settings.toml");
    
    let mut store = SettingsStore::new(store_path.clone());
    store.settings.theme = Some(sway_configurator::model::theme::ThemeSelection {
        name: "test-theme".to_string(),
        source: "test-source".to_string(),
    });
    
    store.save().expect("Failed to save settings");
    assert!(store_path.exists(), "Settings file was not created");
    
    let loaded_store = SettingsStore::load(store_path).expect("Failed to load settings");
    assert_eq!(loaded_store.settings.theme.as_ref().unwrap().name, "test-theme");
    assert_eq!(loaded_store.settings.theme.as_ref().unwrap().source, "test-source");
}

#[test]
fn test_store_load_nonexistent_creates_default() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let store_path = temp_dir.path().join("nonexistent.toml");
    
    let store = SettingsStore::load(store_path).expect("Failed to load nonexistent store");
    assert!(store.settings.theme.is_none(), "Default settings should have no theme");
}

#[test]
fn test_store_preserves_data_across_roundtrip() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let store_path = temp_dir.path().join("settings.toml");
    
    // First store
    {
        let mut store = SettingsStore::new(store_path.clone());
        store.settings.theme = Some(sway_configurator::model::theme::ThemeSelection {
            name: "theme-1".to_string(),
            source: "source-1".to_string(),
        });
        store.save().expect("Failed to save first time");
    }
    
    // Load and verify
    {
        let store = SettingsStore::load(store_path.clone()).expect("Failed to load");
        assert_eq!(store.settings.theme.as_ref().unwrap().name, "theme-1");
    }
    
    // Modify and save again
    {
        let mut store = SettingsStore::load(store_path.clone()).expect("Failed to reload");
        store.settings.theme = Some(sway_configurator::model::theme::ThemeSelection {
            name: "theme-2".to_string(),
            source: "source-2".to_string(),
        });
        store.save().expect("Failed to save second time");
    }
    
    // Load and verify final state
    {
        let store = SettingsStore::load(store_path).expect("Failed to load final");
        assert_eq!(store.settings.theme.as_ref().unwrap().name, "theme-2");
    }
}

#[test]
fn test_open_default_returns_store() {
    // Call open_default - this should not fail
    // It gracefully returns a default store even if the config file doesn't exist
    let store = SettingsStore::open_default().expect("Failed to open default store");
    
    // Verify the store has a valid path that ends with the expected file
    assert!(
        store.path.to_string_lossy().ends_with("sway-configurator/settings.toml"),
        "Store path should end with 'sway-configurator/settings.toml', got: {}",
        store.path.display()
    );
    
    // Verify the store contains default settings
    assert!(store.settings.theme.is_none(), "Default store should have no theme set");
}