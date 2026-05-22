/// Application state management
/// Tracks the current settings and dirty state (whether changes have been made)
use crate::model::settings::Settings;

/// Shared application state tracking settings and dirty status
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current editable settings
    settings: Settings,
    /// Whether settings have been modified but not saved
    dirty: bool,
}

impl AppState {
    /// Create a new AppState with the given settings, initially clean
    pub fn new(settings: Settings) -> Self {
        AppState {
            settings,
            dirty: false,
        }
    }

    /// Mark the state as dirty (settings have changed)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark the state as clean (settings have been saved)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Check if the state is dirty (has unsaved changes)
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get a reference to the current settings
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get a mutable reference to the current settings
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::theme::ThemeSelection;

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
}
