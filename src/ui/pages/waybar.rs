/// Waybar configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;
use crate::model::waybar::{WaybarConfig, BarPosition};

/// Waybar page - for configuring the status bar
pub struct WaybarPage {
    widget: gtk4::Box,
    enabled_switch: libadwaita::SwitchRow,
    position_combo: libadwaita::ComboRow,
    height_spin: libadwaita::SpinRow,
    preferences_group: libadwaita::PreferencesGroup,
    banner_group: libadwaita::PreferencesGroup,
}

impl WaybarPage {
    /// Create a new waybar page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let prefs_page = libadwaita::PreferencesPage::new();
        
        // Main group
        let preferences_group = libadwaita::PreferencesGroup::new();
        preferences_group.set_title("Waybar Configuration");
        
        // Enabled switch
        let enabled_switch = libadwaita::SwitchRow::new();
        enabled_switch.set_title("Enable waybar");
        enabled_switch.set_subtitle("Show the status bar on screen");
        enabled_switch.set_active(true);
        preferences_group.add(&enabled_switch);
        
        // Position combo
        let position_model = gtk4::StringList::new(&["Top", "Bottom", "Left", "Right"]);
        let position_combo = libadwaita::ComboRow::new();
        position_combo.set_model(Some(&position_model));
        position_combo.set_title("Position");
        position_combo.set_subtitle("Position of the bar on screen");
        position_combo.set_selected(0); // Default: Top
        preferences_group.add(&position_combo);
        
        // Height spin row
        let adj = gtk4::Adjustment::new(30.0, 10.0, 200.0, 1.0, 5.0, 0.0);
        let height_spin = libadwaita::SpinRow::new(Some(&adj), 1.0, 0);
        height_spin.set_title("Height");
        height_spin.set_subtitle("Height of the bar in pixels");
        preferences_group.add(&height_spin);
        
        prefs_page.add(&preferences_group);
        
        // Banner group for warnings (starts empty, will be populated by bind_detection)
        let banner_group = libadwaita::PreferencesGroup::new();
        prefs_page.add(&banner_group);
        
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);
        
        WaybarPage {
            widget,
            enabled_switch,
            position_combo,
            height_spin,
            preferences_group,
            banner_group,
        }
    }
    
    /// Load waybar configuration into the page
    pub fn load_waybar(&self, config: &WaybarConfig) {
        self.enabled_switch.set_active(config.enabled);
        
        let pos_idx = match config.position {
            BarPosition::Top => 0,
            BarPosition::Bottom => 1,
            BarPosition::Left => 2,
            BarPosition::Right => 3,
        };
        self.position_combo.set_selected(pos_idx);
        self.height_spin.set_value(config.height as f64);
    }
    
    /// Bind feature detection and show warnings if needed
    pub fn bind_detection(&self, has_waybar: bool) {
        // Clear existing banner warnings
        while let Some(child) = self.banner_group.first_child() {
            self.banner_group.remove(&child);
        }
        
        // Disable preferences group if waybar is not found
        if !has_waybar {
            self.preferences_group.set_sensitive(false);
            
            let banner = libadwaita::ActionRow::new();
            banner.set_title("Warning: Waybar is not installed");
            banner.set_subtitle("Install waybar to use this configuration");
            self.banner_group.add(&banner);
        } else {
            // Waybar available - enable controls and hide warnings
            self.preferences_group.set_sensitive(true);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for WaybarPage {
    fn default() -> Self {
        Self::new()
    }
}
