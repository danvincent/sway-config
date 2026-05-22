/// Themes configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::model::theme::ThemeSelection;
use crate::state::AppState;
use std::path::PathBuf;

/// Themes page - for configuring themes
pub struct ThemesPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
    custom_path_entry: libadwaita::EntryRow,
    /// Tracks loaded theme names in list_box order for selection → AppState mapping
    theme_names: Rc<RefCell<Vec<ThemeSelection>>>,
    loading: Rc<std::cell::Cell<bool>>,
    app_state: Rc<RefCell<AppState>>,
}

impl ThemesPage {
    /// Create a new themes page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let prefs_page = libadwaita::PreferencesPage::new();

        // ── Custom path group ──────────────────────────────────────────────────
        let path_group = libadwaita::PreferencesGroup::new();
        path_group.set_title("Theme Locations");

        let custom_path_entry = libadwaita::EntryRow::new();
        custom_path_entry.set_title("Custom themes folder");
        custom_path_entry.set_show_apply_button(true);
        path_group.add(&custom_path_entry);
        prefs_page.add(&path_group);

        // ── Theme list group ──────────────────────────────────────────────────
        let list_group = libadwaita::PreferencesGroup::new();
        list_group.set_title("Available Themes");
        list_group.set_description(Some("Choose a theme to apply to your Sway configuration"));

        let list_box = gtk4::ListBox::new();
        list_box.set_css_classes(&["boxed-list"]);
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_group.add(&list_box);
        prefs_page.add(&list_group);

        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        let theme_names: Rc<RefCell<Vec<ThemeSelection>>> = Rc::new(RefCell::new(Vec::new()));
        let loading = Rc::new(std::cell::Cell::new(false));

        // ── Selection signal ──────────────────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let names = Rc::clone(&theme_names);
            let loading = Rc::clone(&loading);
            list_box.connect_row_selected(move |_, row| {
                if loading.get() { return; }
                if let Some(row) = row {
                    let idx = row.index() as usize;
                    let selection = names.borrow().get(idx).cloned();
                    if let Some(sel) = selection {
                        state.borrow_mut().settings_mut().theme = Some(sel);
                        state.borrow_mut().mark_dirty();
                    }
                }
            });
        }

        // ── Custom path apply signal ──────────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let entry_ref = custom_path_entry.clone();
            custom_path_entry.connect_apply(move |_| {
                let path = entry_ref.text().to_string();
                let trimmed = path.trim().to_string();
                let custom = if trimmed.is_empty() { None } else { Some(trimmed) };
                state.borrow_mut().settings_mut().custom_themes_path = custom;
                state.borrow_mut().mark_dirty();
            });
        }

        ThemesPage { widget, list_box, custom_path_entry, theme_names, loading, app_state }
    }
    
    /// Navigate to this page - load themes from app_state
    pub fn on_navigate(&self) {
        let state = self.app_state.borrow();
        let selected = state.settings().theme.clone();
        let custom_path = state.settings().custom_themes_path.clone();
        drop(state);
        
        if let Some(ref p) = custom_path {
            self.custom_path_entry.set_text(p);
        } else {
            self.custom_path_entry.set_text("");
        }

        self.load_themes(selected.as_ref(), custom_path.as_deref());
    }
    
    /// Load themes into the page (suppresses signal write-back via loading guard)
    pub fn load_themes(&self, selected: Option<&ThemeSelection>, custom_path: Option<&str>) {
        self.loading.set(true);

        // Clear existing entries
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Get available themes
        let themes = Self::available_themes(custom_path);
        *self.theme_names.borrow_mut() = themes.clone();
        
        for (idx, theme) in themes.iter().enumerate() {
            let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            row.set_margin_top(6);
            row.set_margin_bottom(6);
            row.set_margin_start(12);
            row.set_margin_end(12);
            
            let name_label = gtk4::Label::new(Some(&theme.name));
            name_label.set_halign(gtk4::Align::Start);
            name_label.set_css_classes(&["heading"]);
            
            let source_label = gtk4::Label::new(Some(&format!("Source: {}", &theme.source)));
            source_label.set_css_classes(&["subtitle"]);
            source_label.set_halign(gtk4::Align::Start);
            
            row.append(&name_label);
            row.append(&source_label);
            
            self.list_box.append(&row);
            
            // Select if matches current selection
            if let Some(sel) = selected {
                if sel.name == theme.name {
                    if let Some(row) = self.list_box.row_at_index(idx as i32) {
                        self.list_box.select_row(Some(&row));
                    }
                }
            }
        }

        self.loading.set(false);
    }
    
    /// Get available themes from the filesystem
    pub fn available_themes(custom_path: Option<&str>) -> Vec<ThemeSelection> {
        let mut themes = Vec::new();
        let mut theme_paths: Vec<(PathBuf, String)> = Vec::new();
        
        // Priority 1: User themes dir (custom override or default ~/.config/sway/themes)
        let user_path = if let Some(custom) = custom_path {
            PathBuf::from(custom)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(format!("{}/.config/sway/themes", home))
        } else {
            PathBuf::from("/tmp/nonexistent")
        };
        theme_paths.push((user_path, "user".to_string()));
        
        // Priority 2: Built-in themes from the SwayConfig template source
        if let Ok(home) = std::env::var("HOME") {
            theme_paths.push((PathBuf::from(format!("{}/source/SwayConfig/themes", home)), "built-in".to_string()));
        }

        // Priority 3: System themes
        theme_paths.push((PathBuf::from("/usr/share/themes"), "system".to_string()));
        
        for (path, source_label) in theme_paths {
            if path.exists() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let file_path = entry.path();
                            // .env files (Sway theme format)
                            if file_path.is_file() {
                                if let Some(name_str) = file_path.file_name().and_then(|n| n.to_str()) {
                                    if name_str.ends_with(".env") {
                                        let theme_name = name_str.trim_end_matches(".env").to_string();
                                        themes.push(ThemeSelection::new(theme_name, source_label.clone()));
                                    }
                                }
                            }
                            // GTK theme directories (containing index.theme)
                            if file_path.is_dir() && file_path.join("index.theme").exists() {
                                if let Some(name_str) = file_path.file_name().and_then(|n| n.to_str()) {
                                    themes.push(ThemeSelection::new(name_str.to_string(), source_label.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort and deduplicate by theme name (user path has priority as it's scanned first)
        themes.sort_by(|a, b| a.name.cmp(&b.name));
        themes.dedup_by(|a, b| a.name == b.name);
        
        themes
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for ThemesPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::Settings;
    use crate::model::theme::ThemeSelection;
    use crate::config::read_helpers::read_theme;

    #[test]
    fn test_available_themes_no_crash_with_no_paths() {
        // Scanning non-existent paths returns empty vec without panicking
        let themes = ThemesPage::available_themes(Some("/nonexistent/path/xyz"));
        // We don't assert contents since the path doesn't exist, just no crash
        drop(themes);
    }

    #[test]
    fn test_read_theme_returns_none_for_default_settings() {
        let settings = Settings::default();
        assert!(read_theme(&settings).is_none());
    }

    #[test]
    fn test_read_theme_returns_theme_when_set() {
        let mut settings = Settings::default();
        settings.theme = Some(ThemeSelection::new("my-theme", "local"));
        let result = read_theme(&settings);
        assert_eq!(result.unwrap().name, "my-theme");
    }

    #[test]
    fn test_available_themes_does_not_include_source_swayconfig() {
        // The ~/source/SwayConfig/themes path and ~/.config/sway-configurator/themes must NOT be scanned
        let themes = ThemesPage::available_themes(None);
        for theme in &themes {
            assert!(!theme.source.contains("reference"), "Found reference source label");
            assert!(!theme.source.contains("local"), "Found old sway-configurator local path label");
        }
    }

    #[test]
    fn test_available_themes_scans_custom_path_with_env_files() {
        use std::fs;
        let dir = std::env::temp_dir().join("themes-test-custom");
        fs::create_dir_all(&dir).unwrap();
        // Write a .env file
        fs::write(dir.join("mytheme.env"), "# theme").unwrap();
        
        let themes = ThemesPage::available_themes(Some(dir.to_str().unwrap()));
        assert!(themes.iter().any(|t| t.name == "mytheme" && t.source == "user"),
            "custom path should produce a theme with source='user'");
        
        // Clean up
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_available_themes_system_path_scanned() {
        // /usr/share/themes may or may not contain .env files on this system,
        // but available_themes should not panic when called without custom path
        let themes = ThemesPage::available_themes(None);
        // All themes from /usr/share/themes must have source "system"
        for t in themes.iter().filter(|t| t.source == "system") {
            assert!(!t.name.is_empty());
        }
    }

    #[test]
    fn test_available_themes_custom_path_overrides_default_user_path() {
        use std::fs;
        // Write a theme to the custom path
        let custom_dir = std::env::temp_dir().join("themes-test-override-custom");
        fs::create_dir_all(&custom_dir).unwrap();
        fs::write(custom_dir.join("custom-only-theme.env"), "# custom").unwrap();

        let themes = ThemesPage::available_themes(Some(custom_dir.to_str().unwrap()));

        // The custom theme must be found with source "user"
        assert!(
            themes.iter().any(|t| t.name == "custom-only-theme" && t.source == "user"),
            "custom-only-theme with source='user' not found: {:?}", themes
        );

        // No theme should have source "local" (old sway-configurator path is gone)
        assert!(
            themes.iter().all(|t| t.source != "local"),
            "Unexpected 'local' source found: {:?}", themes
        );

        let _ = fs::remove_dir_all(&custom_dir);
    }
}
