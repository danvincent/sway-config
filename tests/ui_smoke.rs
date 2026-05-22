/// Smoke tests for UI components
/// Tests state management logic (AppState, dirty tracking)
/// Does NOT test GTK widgets directly since they require a display
use sway_configurator::state::AppState;
use sway_configurator::model::settings::Settings;
use sway_configurator::model::theme::ThemeSelection;
use sway_configurator::ui::pages;

#[test]
fn test_app_state_new() {
    let settings = Settings::default();
    let state = AppState::new(settings);
    
    // AppState should start clean (not dirty)
    assert!(!state.is_dirty(), "AppState should not be dirty on creation");
}

#[test]
fn test_app_state_mark_dirty() {
    let settings = Settings::default();
    let mut state = AppState::new(settings);
    
    state.mark_dirty();
    
    assert!(state.is_dirty(), "AppState should be dirty after mark_dirty()");
}

#[test]
fn test_app_state_mark_clean() {
    let settings = Settings::default();
    let mut state = AppState::new(settings);
    
    state.mark_dirty();
    assert!(state.is_dirty(), "AppState should be dirty");
    
    state.mark_clean();
    assert!(!state.is_dirty(), "AppState should not be dirty after mark_clean()");
}

#[test]
fn test_app_state_holds_settings() {
    let mut settings = Settings::default();
    settings.theme = Some(ThemeSelection::new("dark", "system"));
    
    let state = AppState::new(settings.clone());
    
    // Verify settings are stored
    assert_eq!(state.settings().theme.as_ref().unwrap().name, "dark");
}

#[test]
fn test_app_state_multiple_mark_dirty_calls() {
    let settings = Settings::default();
    let mut state = AppState::new(settings);
    
    state.mark_dirty();
    state.mark_dirty();
    state.mark_dirty();
    
    assert!(state.is_dirty(), "AppState should remain dirty after multiple mark_dirty calls");
}

#[test]
fn test_app_state_clean_after_dirty_cycle() {
    let settings = Settings::default();
    let mut state = AppState::new(settings);
    
    state.mark_dirty();
    state.mark_clean();
    state.mark_dirty();
    state.mark_clean();
    
    assert!(!state.is_dirty(), "AppState should be clean after final mark_clean()");
}

// ============ UI Tests for Issue 3 ============

#[test]
fn test_page_ids_are_unique() {
    let ids = pages::page_ids();
    
    // All IDs should be unique
    let mut seen = std::collections::HashSet::new();
    for &id in ids {
        assert!(
            seen.insert(id),
            "Page ID '{}' appears more than once",
            id
        );
    }
}

#[test]
fn test_sidebar_order() {
    let ids = pages::page_ids();
    
    // Must have exactly 7 pages in the correct order
    let expected = &["outputs", "inputs", "idle", "waybar", "autostart", "notifications", "themes"];
    assert_eq!(
        ids, expected,
        "Page IDs must be in the exact required order"
    );
}

#[test]
fn test_apply_bar_dirty_logic() {
    let settings = Settings::default();
    let mut state = AppState::new(settings);
    
    // Initially not dirty
    assert!(!state.is_dirty(), "Should start clean");
    
    // Mark dirty
    state.mark_dirty();
    assert!(state.is_dirty(), "Should be dirty after mark_dirty");
    
    // Mark clean
    state.mark_clean();
    assert!(!state.is_dirty(), "Should be clean after mark_clean");
}

#[test]
fn test_page_ids_valid_values() {
    let ids = pages::page_ids();
    let valid_page_ids = [
        "outputs", "inputs", "idle", "waybar",
        "autostart", "notifications", "themes"
    ];
    
    // All page IDs should be in the valid list
    for &id in ids {
        assert!(
            valid_page_ids.contains(&id),
            "Page ID '{}' is not a valid page",
            id
        );
    }
}
