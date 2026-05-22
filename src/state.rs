/// Application state management
/// Tracks the current settings and dirty state (whether changes have been made)
use crate::model::settings::Settings;

/// Shared application state tracking settings and dirty status
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current editable settings
    settings: Settings,
    /// Whether any settings have been modified but not saved
    dirty: bool,
    /// Per-section dirty flags — used by navigation guards so an edit in one
    /// section never blocks hardware detection refresh in another.
    outputs_dirty: bool,
    keyboards_dirty: bool,
    touchpads_dirty: bool,
}

impl AppState {
    /// Create a new AppState with the given settings, initially clean
    pub fn new(settings: Settings) -> Self {
        AppState {
            settings,
            dirty: false,
            outputs_dirty: false,
            keyboards_dirty: false,
            touchpads_dirty: false,
        }
    }

    /// Mark the state as dirty (settings have changed)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark the state as clean (settings have been saved or reverted)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.outputs_dirty = false;
        self.keyboards_dirty = false;
        self.touchpads_dirty = false;
    }

    /// Check if any settings are dirty (has unsaved changes)
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if outputs specifically have unsaved user edits
    pub fn is_outputs_dirty(&self) -> bool {
        self.outputs_dirty
    }

    /// Check if keyboard settings specifically have unsaved user edits
    pub fn is_keyboards_dirty(&self) -> bool {
        self.keyboards_dirty
    }

    /// Check if touchpad settings specifically have unsaved user edits
    pub fn is_touchpads_dirty(&self) -> bool {
        self.touchpads_dirty
    }

    /// Returns true if hardware detection should refresh outputs.
    /// False when the user has unsaved outputs edits that must be preserved.
    /// Used by OutputsPage::on_navigate() — tested independently of GTK.
    pub fn should_refresh_outputs(&self) -> bool {
        !self.outputs_dirty
    }

    /// Returns true if hardware detection should refresh keyboards.
    /// False when the user has unsaved keyboard edits that must be preserved.
    /// Used by InputsPage::on_navigate() — tested independently of GTK.
    pub fn should_refresh_keyboards(&self) -> bool {
        !self.keyboards_dirty
    }

    /// Returns true if hardware detection should refresh touchpads.
    /// False when the user has unsaved touchpad edits that must be preserved.
    /// Used by InputsPage::on_navigate() — tested independently of GTK.
    pub fn should_refresh_touchpads(&self) -> bool {
        !self.touchpads_dirty
    }

    /// Mark outputs as dirty from an individual-field update.
    /// Use this when updating a single output field via settings_mut() instead of set_outputs().
    pub fn mark_outputs_dirty(&mut self) {
        self.dirty = true;
        self.outputs_dirty = true;
    }

    /// Mark keyboards as dirty from an individual-field update.
    /// Use this when updating a single keyboard field via settings_mut() instead of set_keyboards().
    pub fn mark_keyboards_dirty(&mut self) {
        self.dirty = true;
        self.keyboards_dirty = true;
    }

    /// Mark touchpads as dirty from an individual-field update.
    /// Use this when updating a single touchpad field via settings_mut() instead of set_touchpads().
    pub fn mark_touchpads_dirty(&mut self) {
        self.dirty = true;
        self.touchpads_dirty = true;
    }

    /// Get a reference to the current settings
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get a mutable reference to the current settings
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Update outputs in settings and mark dirty
    pub fn set_outputs(&mut self, outputs: Vec<crate::model::output::OutputConfig>) {
        self.settings.outputs = outputs;
        self.dirty = true;
        self.outputs_dirty = true;
    }

    /// Update keyboards in settings and mark dirty
    pub fn set_keyboards(&mut self, keyboards: Vec<crate::model::input::KeyboardConfig>) {
        self.settings.keyboards = keyboards;
        self.dirty = true;
        self.keyboards_dirty = true;
    }

    /// Update touchpads in settings and mark dirty
    pub fn set_touchpads(&mut self, touchpads: Vec<crate::model::input::TouchpadConfig>) {
        self.settings.touchpads = touchpads;
        self.dirty = true;
        self.touchpads_dirty = true;
    }

    /// Refresh detected outputs — updates settings without marking dirty.
    /// Used for hardware detection on navigation, not user edits.
    pub fn refresh_outputs(&mut self, outputs: Vec<crate::model::output::OutputConfig>) {
        self.settings.outputs = outputs;
    }

    /// Refresh detected keyboards — updates settings without marking dirty.
    /// Used for hardware detection on navigation, not user edits.
    pub fn refresh_keyboards(&mut self, keyboards: Vec<crate::model::input::KeyboardConfig>) {
        self.settings.keyboards = keyboards;
    }

    /// Refresh detected touchpads — updates settings without marking dirty.
    /// Used for hardware detection on navigation, not user edits.
    pub fn refresh_touchpads(&mut self, touchpads: Vec<crate::model::input::TouchpadConfig>) {
        self.settings.touchpads = touchpads;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::theme::ThemeSelection;
    use crate::model::output::OutputConfig;
    use crate::model::input::KeyboardConfig;
    use crate::model::input::TouchpadConfig;

    #[test]
    fn test_app_state_default() {
        let settings = Settings::default();
        let state = AppState::new(settings);
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_app_state_mark_dirty() {
        let settings = Settings::default();
        let mut state = AppState::new(settings);
        state.mark_dirty();
        assert!(state.is_dirty());
    }

    #[test]
    fn test_app_state_mark_clean() {
        let settings = Settings::default();
        let mut state = AppState::new(settings);
        state.mark_dirty();
        state.mark_clean();
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_app_state_settings_access() {
        let mut settings = Settings::default();
        settings.theme = Some(ThemeSelection::new("dark", "system"));
        
        let state = AppState::new(settings);
        assert_eq!(state.settings().theme.as_ref().unwrap().name, "dark");
    }

    #[test]
    fn test_app_state_settings_mut_access() {
        let settings = Settings::default();
        let mut state = AppState::new(settings);
        
        state.settings_mut().theme = Some(ThemeSelection::new("light", "custom"));
        assert_eq!(state.settings().theme.as_ref().unwrap().name, "light");
    }

    #[test]
    fn test_set_outputs_updates_settings_and_marks_dirty() {
        let mut state = AppState::new(Settings::default());
        assert!(!state.is_dirty());
        
        let output = OutputConfig {
            name: "HDMI-1".to_string(),
            enabled: true,
            resolution: None,
            refresh_rate: None,
            position: crate::model::output::Position { x: 0, y: 0 },
            scale: 1.0,
            transform: crate::model::output::Transform::Normal,
        };
        state.set_outputs(vec![output]);
        
        assert!(state.is_dirty());
        assert_eq!(state.settings().outputs.len(), 1);
    }

    #[test]
    fn test_set_keyboards_updates_settings_and_marks_dirty() {
        let mut state = AppState::new(Settings::default());
        
        let keyboard = KeyboardConfig {
            identifier: "kbd".to_string(),
            xkb_layout: "us".to_string(),
            xkb_variant: String::new(),
            xkb_options: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
        };
        state.set_keyboards(vec![keyboard]);
        
        assert!(state.is_dirty());
        assert_eq!(state.settings().keyboards.len(), 1);
    }

    #[test]
    fn test_set_touchpads_updates_settings_and_marks_dirty() {
        let mut state = AppState::new(Settings::default());
        
        let touchpad = TouchpadConfig {
            identifier: "pad".to_string(),
            tap_to_click: false,
            natural_scroll: false,
            dwt: false,
            accel_speed: 0.0,
            accel_profile: crate::model::input::AccelProfile::Adaptive,
            left_handed: false,
            middle_emulation: false,
        };
        state.set_touchpads(vec![touchpad]);
        
        assert!(state.is_dirty());
        assert_eq!(state.settings().touchpads.len(), 1);
    }

    #[test]
    fn test_refresh_outputs_does_not_mark_dirty() {
        let mut state = AppState::new(Settings::default());
        state.refresh_outputs(vec![OutputConfig {
            name: "HDMI-1".to_string(),
            enabled: true,
            resolution: None,
            refresh_rate: None,
            position: crate::model::output::Position { x: 0, y: 0 },
            scale: 1.0,
            transform: crate::model::output::Transform::Normal,
        }]);
        assert!(!state.is_dirty(), "refresh_outputs should not mark dirty");
        assert_eq!(state.settings().outputs.len(), 1);
    }

    #[test]
    fn test_refresh_keyboards_does_not_mark_dirty() {
        let mut state = AppState::new(Settings::default());
        state.refresh_keyboards(vec![KeyboardConfig {
            identifier: "kbd".to_string(),
            xkb_layout: "us".to_string(),
            xkb_variant: String::new(),
            xkb_options: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
        }]);
        assert!(!state.is_dirty(), "refresh_keyboards should not mark dirty");
        assert_eq!(state.settings().keyboards.len(), 1);
    }

    #[test]
    fn test_refresh_touchpads_does_not_mark_dirty() {
        let mut state = AppState::new(Settings::default());
        state.refresh_touchpads(vec![TouchpadConfig {
            identifier: "pad".to_string(),
            tap_to_click: false,
            natural_scroll: false,
            dwt: false,
            accel_speed: 0.0,
            accel_profile: crate::model::input::AccelProfile::Adaptive,
            left_handed: false,
            middle_emulation: false,
        }]);
        assert!(!state.is_dirty(), "refresh_touchpads should not mark dirty");
        assert_eq!(state.settings().touchpads.len(), 1);
    }

    #[test]
    fn test_set_outputs_marks_outputs_dirty_only() {
        let mut state = AppState::new(Settings::default());
        let output = OutputConfig {
            name: "DP-1".to_string(),
            enabled: true,
            resolution: None,
            refresh_rate: None,
            position: crate::model::output::Position { x: 0, y: 0 },
            scale: 1.0,
            transform: crate::model::output::Transform::Normal,
        };
        state.set_outputs(vec![output]);

        assert!(state.is_dirty());
        assert!(state.is_outputs_dirty());
        assert!(!state.is_keyboards_dirty(), "keyboards should not be dirty");
        assert!(!state.is_touchpads_dirty(), "touchpads should not be dirty");
    }

    #[test]
    fn test_keyboards_dirty_does_not_affect_outputs_guard() {
        // Editing keyboards should not cause the outputs detection guard to fire.
        // This verifies the production logic: on_navigate calls should_refresh_outputs(),
        // not the global is_dirty(), so unrelated edits don't suppress detection.
        let mut state = AppState::new(Settings::default());
        let keyboard = KeyboardConfig {
            identifier: "kbd".to_string(),
            xkb_layout: "gb".to_string(),
            xkb_variant: String::new(),
            xkb_options: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
        };
        state.set_keyboards(vec![keyboard]);

        assert!(state.is_dirty(), "global dirty set");
        assert!(state.is_keyboards_dirty());
        assert!(!state.is_outputs_dirty(), "outputs must still be clean — detection should proceed");
    }

    #[test]
    fn test_outputs_dirty_does_not_affect_inputs_guard() {
        let mut state = AppState::new(Settings::default());
        let output = OutputConfig {
            name: "HDMI-1".to_string(),
            enabled: true,
            resolution: None,
            refresh_rate: None,
            position: crate::model::output::Position { x: 0, y: 0 },
            scale: 1.0,
            transform: crate::model::output::Transform::Normal,
        };
        state.set_outputs(vec![output]);

        assert!(state.is_dirty(), "global dirty set");
        assert!(state.is_outputs_dirty());
        assert!(!state.is_keyboards_dirty(), "keyboards must still be clean — inputs detection should proceed");
        assert!(!state.is_touchpads_dirty(), "touchpads must still be clean — inputs detection should proceed");
    }

    #[test]
    fn test_mark_clean_clears_all_section_flags() {
        let mut state = AppState::new(Settings::default());
        let output = OutputConfig {
            name: "DP-1".to_string(),
            enabled: true,
            resolution: None,
            refresh_rate: None,
            position: crate::model::output::Position { x: 0, y: 0 },
            scale: 1.0,
            transform: crate::model::output::Transform::Normal,
        };
        state.set_outputs(vec![output]);
        state.set_keyboards(vec![KeyboardConfig {
            identifier: "kbd".to_string(),
            xkb_layout: "gb".to_string(),
            xkb_variant: String::new(),
            xkb_options: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
        }]);
        state.mark_clean();

        assert!(!state.is_dirty());
        assert!(!state.is_outputs_dirty());
        assert!(!state.is_keyboards_dirty());
        assert!(!state.is_touchpads_dirty());
    }

    #[test]
    fn test_mark_outputs_dirty_sets_both_global_and_section() {
        let mut state = AppState::new(Settings::default());
        assert!(!state.is_dirty());
        assert!(!state.is_outputs_dirty());

        state.mark_outputs_dirty();

        assert!(state.is_dirty(), "global dirty must be set");
        assert!(state.is_outputs_dirty(), "outputs section dirty must be set");
        assert!(!state.is_keyboards_dirty(), "keyboards must be unaffected");
        assert!(!state.is_touchpads_dirty(), "touchpads must be unaffected");
    }

    #[test]
    fn test_individual_output_field_update_via_settings_mut() {
        // Simulates what the ExpanderRow signal closures do: update a single field
        // through settings_mut() then call mark_outputs_dirty().
        let mut state = AppState::new(Settings::default());
        state.set_outputs(vec![OutputConfig {
            name: "HDMI-1".to_string(),
            enabled: true,
            resolution: None,
            refresh_rate: None,
            position: crate::model::output::Position { x: 0, y: 0 },
            scale: 1.0,
            transform: crate::model::output::Transform::Normal,
        }]);
        state.mark_clean(); // simulate starting from a loaded state

        // Now simulate a single-field update (scale)
        if let Some(out) = state.settings_mut().outputs.get_mut(0) {
            out.scale = 2.0;
        }
        state.mark_outputs_dirty();

        assert!(state.is_dirty());
        assert!(state.is_outputs_dirty());
        assert_eq!(state.settings().outputs[0].scale, 2.0);
    }
}
