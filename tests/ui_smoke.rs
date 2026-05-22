/// Smoke tests for UI components
/// Tests state management logic (AppState, dirty tracking)
/// Does NOT test GTK widgets directly since they require a display
use sway_configurator::state::AppState;
use sway_configurator::model::settings::Settings;
use sway_configurator::model::theme::ThemeSelection;
use sway_configurator::model::output::OutputConfig;
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
    
    // Must have exactly 8 pages in the correct order
    let expected = &["outputs", "inputs", "idle", "waybar", "autostart", "notifications", "themes", "general"];
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
        "autostart", "notifications", "themes", "general"
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

// ============ Tests for Fix 1: refresh_* methods ============

#[test]
fn test_refresh_does_not_dirty_state() {
    let mut state = AppState::new(Settings::default());
    state.refresh_outputs(vec![]);
    assert!(!state.is_dirty(), "refresh_outputs should not mark state dirty");
}

#[test]
fn test_set_outputs_dirties_state() {
    let mut state = AppState::new(Settings::default());
    state.set_outputs(vec![]);
    assert!(state.is_dirty(), "set_outputs should mark state dirty");
}

#[test]
fn test_refresh_outputs_updates_settings() {
    let mut state = AppState::new(Settings::default());
    let output = OutputConfig {
        name: "HDMI-1".to_string(),
        enabled: true,
        resolution: None,
        refresh_rate: None,
        position: sway_configurator::model::output::Position { x: 0, y: 0 },
        scale: 1.0,
        transform: sway_configurator::model::output::Transform::Normal,
    };
    state.refresh_outputs(vec![output.clone()]);
    assert_eq!(state.settings().outputs.len(), 1);
    assert_eq!(state.settings().outputs[0].name, "HDMI-1");
    assert!(!state.is_dirty(), "refresh_outputs should not mark dirty");
}

// ============ Section-specific detection guard tests ============
// These verify the helpers used by on_navigate() guards in outputs.rs and inputs.rs.
// OutputsPage calls should_refresh_outputs(); InputsPage calls
// should_refresh_keyboards() / should_refresh_touchpads(). The helpers are
// tested here directly so a regression to is_dirty() would be caught.

#[test]
fn test_outputs_guard_passes_when_keyboards_dirty() {
    // Editing keyboards must not suppress outputs detection.
    let mut state = AppState::new(Settings::default());
    state.set_keyboards(vec![sway_configurator::model::input::KeyboardConfig {
        identifier: "kbd".to_string(),
        xkb_layout: "gb".to_string(),
        xkb_variant: String::new(),
        xkb_options: String::new(),
        repeat_delay: 600,
        repeat_rate: 25,
    }]);
    assert!(state.is_dirty(), "global dirty after keyboard edit");
    assert!(state.should_refresh_outputs(), "outputs detection must still proceed");
}

#[test]
fn test_outputs_guard_blocks_when_outputs_dirty() {
    // When the user has edited outputs the guard must block detection to
    // preserve their edits. should_refresh_outputs() returns false.
    let mut state = AppState::new(Settings::default());
    state.set_outputs(vec![OutputConfig {
        name: "user-edit".to_string(),
        enabled: true,
        resolution: None,
        refresh_rate: None,
        position: sway_configurator::model::output::Position { x: 0, y: 0 },
        scale: 1.0,
        transform: sway_configurator::model::output::Transform::Normal,
    }]);
    assert!(!state.should_refresh_outputs(), "outputs guard must block detection when outputs are dirty");
}

#[test]
fn test_keyboards_guard_passes_when_touchpads_dirty() {
    // Editing touchpads must not suppress keyboard detection.
    let mut state = AppState::new(Settings::default());
    state.set_touchpads(vec![sway_configurator::model::input::TouchpadConfig {
        identifier: "pad".to_string(),
        tap_to_click: true,
        natural_scroll: false,
        dwt: false,
        accel_speed: 0.0,
        accel_profile: sway_configurator::model::input::AccelProfile::Adaptive,
        left_handed: false,
        middle_emulation: false,
    }]);
    assert!(state.should_refresh_keyboards(), "keyboard detection must still proceed when only touchpads are dirty");
}

#[test]
fn test_touchpads_guard_passes_when_keyboards_dirty() {
    // Editing keyboards must not suppress touchpad detection.
    let mut state = AppState::new(Settings::default());
    state.set_keyboards(vec![sway_configurator::model::input::KeyboardConfig {
        identifier: "kbd".to_string(),
        xkb_layout: "gb".to_string(),
        xkb_variant: String::new(),
        xkb_options: String::new(),
        repeat_delay: 600,
        repeat_rate: 25,
    }]);
    assert!(state.should_refresh_touchpads(), "touchpad detection must still proceed when only keyboards are dirty");
}

#[test]
fn test_inputs_guard_passes_when_outputs_dirty() {
    // An edit to outputs must not suppress inputs detection.
    let mut state = AppState::new(Settings::default());
    state.set_outputs(vec![OutputConfig {
        name: "DP-1".to_string(),
        enabled: true,
        resolution: None,
        refresh_rate: None,
        position: sway_configurator::model::output::Position { x: 0, y: 0 },
        scale: 1.0,
        transform: sway_configurator::model::output::Transform::Normal,
    }]);
    assert!(state.should_refresh_keyboards(), "keyboard detection must proceed when only outputs are dirty");
    assert!(state.should_refresh_touchpads(), "touchpad detection must proceed when only outputs are dirty");
}

#[test]
fn test_mark_clean_restores_all_detection_guards() {
    let mut state = AppState::new(Settings::default());
    state.set_outputs(vec![]);
    state.set_keyboards(vec![]);
    state.mark_clean();
    assert!(state.should_refresh_outputs(), "outputs detection must be enabled after clean");
    assert!(state.should_refresh_keyboards(), "keyboard detection must be enabled after clean");
    assert!(state.should_refresh_touchpads(), "touchpad detection must be enabled after clean");
}

// ============ General page test ============

#[test]
fn test_general_page_reads_terminal_from_settings() {
    use sway_configurator::config::read_helpers::read_general;
    
    let mut settings = Settings::default();
    settings.general.terminal = "alacritty".to_string();
    let config = read_general(&settings);
    assert_eq!(config.terminal, "alacritty");
}

// ============ Phase 3: Outputs page helper tests ============
// These test the pure (no-GTK) helper functions used by the outputs ExpanderRow UI.

#[test]
fn test_format_mode_60hz() {
    use sway_configurator::ui::pages::outputs::format_mode;
    assert_eq!(format_mode(1920, 1080, 60000), "1920×1080 @ 60Hz");
}

#[test]
fn test_format_mode_144hz() {
    use sway_configurator::ui::pages::outputs::format_mode;
    assert_eq!(format_mode(2560, 1440, 144000), "2560×1440 @ 144Hz");
}

#[test]
fn test_transform_labels_all_distinct() {
    use sway_configurator::ui::pages::outputs::{TRANSFORMS, transform_label};
    let labels: Vec<&str> = TRANSFORMS.iter().map(|t| transform_label(*t)).collect();
    let mut unique = labels.clone();
    unique.dedup();
    assert_eq!(labels.len(), unique.len(), "all transform labels must be unique");
}

#[test]
fn test_transform_index_roundtrip() {
    use sway_configurator::ui::pages::outputs::{TRANSFORMS, transform_index, transform_from_index};
    for &t in TRANSFORMS {
        let idx = transform_index(t);
        assert_eq!(transform_from_index(idx), t, "roundtrip failed for {:?}", t);
    }
}

#[test]
fn test_transform_label_normal() {
    use sway_configurator::ui::pages::outputs::transform_label;
    use sway_configurator::model::output::Transform;
    assert_eq!(transform_label(Transform::Normal), "Normal");
    assert_eq!(transform_label(Transform::Rotate90), "90°");
    assert_eq!(transform_label(Transform::Flipped), "Flipped");
}

#[test]
fn test_outputs_config_field_update_marks_dirty() {
    use sway_configurator::model::output::{Position, Transform};
    let mut state = AppState::new(Settings::default());
    state.set_outputs(vec![OutputConfig {
        name: "DP-1".to_string(),
        enabled: true,
        resolution: None,
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    }]);
    state.mark_clean();

    // Simulate what the transform ComboRow closure does
    if let Some(out) = state.settings_mut().outputs.get_mut(0) {
        out.transform = Transform::Rotate90;
    }
    state.mark_outputs_dirty();

    assert!(state.is_dirty());
    assert!(state.is_outputs_dirty());
    assert_eq!(state.settings().outputs[0].transform, Transform::Rotate90);
}

#[test]
fn test_outputs_config_position_update() {
    use sway_configurator::model::output::{Position, Transform};
    let mut state = AppState::new(Settings::default());
    state.set_outputs(vec![OutputConfig {
        name: "HDMI-1".to_string(),
        enabled: true,
        resolution: None,
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    }]);
    state.mark_clean();

    if let Some(out) = state.settings_mut().outputs.get_mut(0) {
        out.position.x = 1920;
        out.position.y = 0;
    }
    state.mark_outputs_dirty();

    assert_eq!(state.settings().outputs[0].position.x, 1920);
    assert!(state.is_outputs_dirty());
}

// ============ Phase 4: Inputs page helper tests ============

#[test]
fn test_accel_profile_labels_distinct() {
    use sway_configurator::ui::pages::inputs::{ACCEL_PROFILES, accel_profile_label};
    let labels: Vec<&str> = ACCEL_PROFILES.iter().map(|&p| accel_profile_label(p)).collect();
    let mut unique = labels.clone();
    unique.dedup();
    assert_eq!(labels.len(), unique.len(), "all accel profile labels must be distinct");
}

#[test]
fn test_accel_profile_index_roundtrip() {
    use sway_configurator::ui::pages::inputs::{ACCEL_PROFILES, accel_profile_index, accel_profile_from_index};
    for &p in &ACCEL_PROFILES {
        let idx = accel_profile_index(p);
        assert_eq!(accel_profile_from_index(idx), p, "roundtrip failed for {:?}", p);
    }
}

#[test]
fn test_keyboard_layout_fallback_from_etc_default_keyboard() {
    use sway_configurator::config::detect::system_keyboard_layout_from_path;
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "XKBLAYOUT=\"gb\"").unwrap();
    writeln!(tmp, "XKBVARIANT=\"\"").unwrap();

    let (layout, variant) = system_keyboard_layout_from_path(tmp.path().to_str().unwrap());
    assert_eq!(layout, "gb");
    assert_eq!(variant, "");
}

#[test]
fn test_keyboard_layout_fallback_missing_file() {
    use sway_configurator::config::detect::system_keyboard_layout_from_path;
    let (layout, _variant) = system_keyboard_layout_from_path("/nonexistent/path/keyboard");
    assert_eq!(layout, "us", "missing file should default to 'us'");
}

#[test]
fn test_keyboard_xkb_override_updates_appstate() {
    use sway_configurator::model::input::KeyboardConfig;
    let mut state = AppState::new(Settings::default());
    state.refresh_keyboards(vec![KeyboardConfig {
        identifier: "1:1:kb".to_string(),
        xkb_layout: "us".to_string(),
        xkb_variant: String::new(),
        xkb_options: String::new(),
        repeat_delay: 600,
        repeat_rate: 25,
    }]);
    assert!(!state.is_keyboards_dirty());

    if let Some(kb) = state.settings_mut().keyboards.get_mut(0) {
        kb.xkb_layout = "gb".to_string();
    }
    state.mark_keyboards_dirty();

    assert_eq!(state.settings().keyboards[0].xkb_layout, "gb");
    assert!(state.is_dirty());
    assert!(state.is_keyboards_dirty());
    // Detection should be suppressed while dirty
    assert!(!state.should_refresh_keyboards());
}

#[test]
fn test_touchpad_tap_toggle_updates_appstate() {
    use sway_configurator::model::input::{AccelProfile, TouchpadConfig};
    let mut state = AppState::new(Settings::default());
    state.refresh_touchpads(vec![TouchpadConfig {
        identifier: "2:7:touchpad".to_string(),
        tap_to_click: false,
        natural_scroll: false,
        dwt: false,
        accel_speed: 0.0,
        accel_profile: AccelProfile::Adaptive,
        left_handed: false,
        middle_emulation: false,
    }]);
    assert!(!state.is_touchpads_dirty());

    if let Some(tp) = state.settings_mut().touchpads.get_mut(0) {
        tp.tap_to_click = true;
    }
    state.mark_touchpads_dirty();

    assert!(state.settings().touchpads[0].tap_to_click);
    assert!(state.is_dirty());
    assert!(state.is_touchpads_dirty());
    assert!(!state.should_refresh_touchpads());
}

#[test]
fn test_touchpad_accel_change_updates_appstate() {
    use sway_configurator::model::input::{AccelProfile, TouchpadConfig};
    let mut state = AppState::new(Settings::default());
    state.refresh_touchpads(vec![TouchpadConfig {
        identifier: "2:7:touchpad".to_string(),
        tap_to_click: false,
        natural_scroll: false,
        dwt: false,
        accel_speed: 0.0,
        accel_profile: AccelProfile::Adaptive,
        left_handed: false,
        middle_emulation: false,
    }]);

    if let Some(tp) = state.settings_mut().touchpads.get_mut(0) {
        tp.accel_speed = 0.5;
        tp.accel_profile = AccelProfile::Flat;
    }
    state.mark_touchpads_dirty();

    assert!((state.settings().touchpads[0].accel_speed - 0.5).abs() < 1e-9);
    assert_eq!(state.settings().touchpads[0].accel_profile, AccelProfile::Flat);
}

#[test]
fn test_inputs_read_back_matches_loaded() {
    use sway_configurator::model::input::{AccelProfile, KeyboardConfig, TouchpadConfig};
    let mut state = AppState::new(Settings::default());

    let kb = KeyboardConfig {
        identifier: "kb1".to_string(),
        xkb_layout: "de".to_string(),
        xkb_variant: "nodeadkeys".to_string(),
        xkb_options: "caps:escape".to_string(),
        repeat_delay: 400,
        repeat_rate: 30,
    };
    let tp = TouchpadConfig {
        identifier: "tp1".to_string(),
        tap_to_click: true,
        natural_scroll: true,
        dwt: true,
        accel_speed: -0.2,
        accel_profile: AccelProfile::Flat,
        left_handed: false,
        middle_emulation: true,
    };

    state.set_keyboards(vec![kb.clone()]);
    state.set_touchpads(vec![tp.clone()]);

    assert_eq!(state.settings().keyboards[0], kb);
    assert_eq!(state.settings().touchpads[0], tp);
}

// ── Phase 5: Idle / Waybar / Notifications / Themes ───────────────────────────

#[test]
fn test_idle_lock_timeout_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().idle.lock_timeout = 600;
    state.mark_dirty();
    assert_eq!(state.settings().idle.lock_timeout, 600);
    assert!(state.is_dirty());
}

#[test]
fn test_idle_screen_off_timeout_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().idle.screen_off_timeout = 120;
    state.mark_dirty();
    assert_eq!(state.settings().idle.screen_off_timeout, 120);
}

#[test]
fn test_idle_lock_command_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().idle.lock_command = "waylock".to_string();
    state.mark_dirty();
    assert_eq!(state.settings().idle.lock_command, "waylock");
}

#[test]
fn test_idle_before_sleep_toggle_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().idle.before_sleep = false;
    state.mark_dirty();
    assert!(!state.settings().idle.before_sleep);
}

#[test]
fn test_waybar_enabled_toggle_updates_appstate() {
    use sway_configurator::ui::pages::waybar::bar_position_from_index;
    let mut state = AppState::new(Settings::default());
    state.settings_mut().waybar.enabled = false;
    state.mark_dirty();
    assert!(!state.settings().waybar.enabled);
    // Ensure position round-trips
    state.settings_mut().waybar.position = bar_position_from_index(1);
    assert_eq!(
        sway_configurator::ui::pages::waybar::bar_position_index(state.settings().waybar.position),
        1
    );
}

#[test]
fn test_waybar_position_index_roundtrip() {
    use sway_configurator::ui::pages::waybar::{bar_position_index, bar_position_from_index};
    use sway_configurator::model::waybar::BarPosition;
    for (idx, pos) in [
        (0, BarPosition::Top),
        (1, BarPosition::Bottom),
        (2, BarPosition::Left),
        (3, BarPosition::Right),
    ] {
        assert_eq!(bar_position_index(pos), idx);
        assert_eq!(bar_position_from_index(idx), pos);
    }
}

#[test]
fn test_waybar_modules_right_selection_updates_appstate() {
    use sway_configurator::model::waybar::WaybarModule;
    let mut state = AppState::new(Settings::default());
    state.settings_mut().waybar.modules_right.push(WaybarModule::new("battery", true));
    state.mark_dirty();
    assert!(state.settings().waybar.modules_right.iter().any(|m| m.name == "battery" && m.enabled));
}

#[test]
fn test_waybar_modules_left_updated() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().waybar.modules_left.push("sway/workspaces".to_string());
    assert!(state.settings().waybar.modules_left.contains(&"sway/workspaces".to_string()));
}

#[test]
fn test_notifications_timeout_updates_appstate() {
    use sway_configurator::ui::pages::notifications::notif_position_index;
    use sway_configurator::model::notifications::NotifPosition;
    let mut state = AppState::new(Settings::default());
    state.settings_mut().notifications.timeout_ms = 3000;
    state.mark_dirty();
    assert_eq!(state.settings().notifications.timeout_ms, 3000);
    assert_eq!(notif_position_index(NotifPosition::TopRight), 0);
}

#[test]
fn test_notifications_position_index_roundtrip() {
    use sway_configurator::ui::pages::notifications::{
        notif_position_index, notif_position_from_index, NOTIF_POSITIONS,
    };
    for (i, &pos) in NOTIF_POSITIONS.iter().enumerate() {
        assert_eq!(notif_position_index(pos), i as u32);
        assert_eq!(notif_position_from_index(i as u32), pos);
    }
}

#[test]
fn test_notifications_max_visible_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().notifications.max_visible = 10;
    state.mark_dirty();
    assert_eq!(state.settings().notifications.max_visible, 10);
}

#[test]
fn test_notifications_follow_focus_toggle_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().notifications.follow_focus = true;
    state.mark_dirty();
    assert!(state.settings().notifications.follow_focus);
}

#[test]
fn test_themes_selection_updates_appstate() {
    use sway_configurator::model::theme::ThemeSelection;
    let mut state = AppState::new(Settings::default());
    state.settings_mut().theme = Some(ThemeSelection::new("dracula", "user"));
    state.mark_dirty();
    let theme = state.settings().theme.as_ref().unwrap();
    assert_eq!(theme.name, "dracula");
    assert_eq!(theme.source, "user");
}

#[test]
fn test_themes_custom_path_updates_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().custom_themes_path = Some("/home/user/.config/sway/themes".to_string());
    state.mark_dirty();
    assert_eq!(
        state.settings().custom_themes_path.as_deref(),
        Some("/home/user/.config/sway/themes")
    );
}

#[test]
fn test_themes_clearing_custom_path_clears_appstate() {
    let mut state = AppState::new(Settings::default());
    state.settings_mut().custom_themes_path = Some("/some/path".to_string());
    state.settings_mut().custom_themes_path = None;
    assert!(state.settings().custom_themes_path.is_none());
}

// ── Phase 5 module toggle semantics + state contract ──────────────────────────
//
// Note: GTK signal handlers (connect_active_notify, connect_row_selected, etc.)
// and the loading guard behaviour in load_idle/load_waybar/load_notifications/
// load_themes require an initialised GTK display and cannot be executed in
// headless `--no-default-features` tests. The tests below cover:
//   (a) the AppState data-model contract that the closures rely on, and
//   (b) the module-list toggle logic that is replicated verbatim in the
//       waybar.rs signal closures.
// Any regression in the guard wiring itself would show up at runtime or in a
// GTK integration test.

/// Baseline contract: `settings_mut()` does not call `mark_dirty()`.
/// Every page's `load_*()` method relies on this — it writes fields without
/// calling `mark_dirty()` and therefore must not dirty the state.
#[test]
fn test_settings_mut_alone_does_not_dirty_state() {
    let mut state = AppState::new(Settings::default());
    state.mark_clean();
    state.settings_mut().idle.lock_timeout = 300;
    state.settings_mut().waybar.height = 42;
    state.settings_mut().notifications.timeout_ms = 1234;
    assert!(!state.is_dirty(), "settings_mut() without mark_dirty() must not dirty state");
}

#[test]
fn test_waybar_module_left_toggle_on_adds_to_vec() {
    let mut state = AppState::new(Settings::default());
    let name = "sway/workspaces".to_string();
    // Mirrors the closure body in build_module_section() for Left:
    state.settings_mut().waybar.modules_left.retain(|m| m != &name);
    state.settings_mut().waybar.modules_left.push(name.clone());
    state.mark_dirty();
    assert!(state.settings().waybar.modules_left.contains(&name));
    assert!(state.is_dirty());
}

#[test]
fn test_waybar_module_left_toggle_off_removes_from_vec() {
    let mut state = AppState::new(Settings::default());
    let name = "sway/workspaces".to_string();
    state.settings_mut().waybar.modules_left.push(name.clone());
    // Mirrors closure body for Left toggle-off:
    state.settings_mut().waybar.modules_left.retain(|m| m != &name);
    state.mark_dirty();
    assert!(!state.settings().waybar.modules_left.contains(&name));
}

#[test]
fn test_waybar_module_right_toggle_on_enables_existing() {
    use sway_configurator::model::waybar::WaybarModule;
    let mut state = AppState::new(Settings::default());
    let name = "battery".to_string();
    state.settings_mut().waybar.modules_right.push(WaybarModule::new(&name, false));
    // Mirrors closure body for Right toggle-on (existing entry):
    let modules = &mut state.settings_mut().waybar.modules_right;
    if let Some(m) = modules.iter_mut().find(|m| m.name == name) {
        m.enabled = true;
    }
    state.mark_dirty();
    assert!(state.settings().waybar.modules_right.iter().any(|m| m.name == name && m.enabled));
}

#[test]
fn test_waybar_module_right_toggle_off_disables_existing() {
    use sway_configurator::model::waybar::WaybarModule;
    let mut state = AppState::new(Settings::default());
    let name = "tray".to_string();
    state.settings_mut().waybar.modules_right.push(WaybarModule::new(&name, true));
    // Mirrors closure body for Right toggle-off:
    let modules = &mut state.settings_mut().waybar.modules_right;
    if let Some(m) = modules.iter_mut().find(|m| m.name == name) {
        m.enabled = false;
    }
    state.mark_dirty();
    assert!(state.settings().waybar.modules_right.iter().any(|m| m.name == name && !m.enabled));
}

#[test]
fn test_waybar_module_right_toggle_on_adds_if_absent() {
    use sway_configurator::model::waybar::WaybarModule;
    let mut state = AppState::new(Settings::default());
    let name = "cpu".to_string();
    // Mirrors closure body for Right toggle-on (absent entry):
    let modules = &mut state.settings_mut().waybar.modules_right;
    if modules.iter_mut().find(|m| m.name == name).is_none() {
        modules.push(WaybarModule::new(&name, true));
    }
    state.mark_dirty();
    assert!(state.settings().waybar.modules_right.iter().any(|m| m.name == name && m.enabled));
}
