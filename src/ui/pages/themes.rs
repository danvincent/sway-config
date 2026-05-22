/// Themes configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;
use crate::model::theme::ThemeSelection;
use std::path::PathBuf;

/// Themes page - for configuring themes
pub struct ThemesPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
}

impl ThemesPage {
    /// Create a new themes page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        
        let title_label = gtk4::Label::new(Some("Select Theme"));
        title_label.set_css_classes(&["title-2"]);
        title_label.set_halign(gtk4::Align::Start);
        vbox.append(&title_label);
        
        let subtitle = gtk4::Label::new(Some("Choose a theme to apply to your sway configuration"));
        subtitle.set_css_classes(&["subtitle"]);
        subtitle.set_halign(gtk4::Align::Start);
        vbox.append(&subtitle);
        
        // List box for themes
        let list_box = gtk4::ListBox::new();
        list_box.set_css_classes(&["boxed-list"]);
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        vbox.append(&list_box);
        
        clamp.set_child(Some(&vbox));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);
        
        ThemesPage { widget, list_box }
    }
    
    /// Load themes into the page
    pub fn load_themes(&self, selected: Option<&ThemeSelection>) {
        // Clear existing entries
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Get available themes
        let themes = Self::available_themes();
        
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
    }
    
    /// Get available themes from the filesystem
    pub fn available_themes() -> Vec<ThemeSelection> {
        let mut themes = Vec::new();
        
        // Get home directory safely
        let mut theme_paths = Vec::new();
        
        if let Ok(home) = std::env::var("HOME") {
            // Priority 1: Custom themes in ~/.config/sway-configurator/themes
            theme_paths.push(PathBuf::from(format!("{}/.config/sway-configurator/themes", home)));
            // Priority 2: Reference SwayConfig themes in ~/source/SwayConfig/themes
            theme_paths.push(PathBuf::from(format!("{}/source/SwayConfig/themes", home)));
        }
        
        // Priority 3: System themes in /usr/share/themes
        theme_paths.push(PathBuf::from("/usr/share/themes"));
        
        for path in theme_paths {
            if path.exists() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let file_path = entry.path();
                            if let Some(file_name) = file_path.file_name() {
                                if let Some(name_str) = file_name.to_str() {
                                    if name_str.ends_with(".env") {
                                        let theme_name = name_str.trim_end_matches(".env").to_string();
                                        let source = if path.to_string_lossy().contains("sway-configurator/themes") {
                                            "custom".to_string()
                                        } else if path.to_string_lossy().contains("source/SwayConfig/themes") {
                                            "reference".to_string()
                                        } else {
                                            "system".to_string()
                                        };
                                        
                                        themes.push(ThemeSelection::new(theme_name, source));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort and deduplicate by theme name (keeping first found, which is highest priority)
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
        Self::new()
    }
}
