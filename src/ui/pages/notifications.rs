/// Notifications configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;
use crate::model::notifications::{NotificationsConfig, NotifPosition};

/// Notifications page - for configuring notifications
pub struct NotificationsPage {
    widget: gtk4::Box,
    timeout_spin: libadwaita::SpinRow,
    max_visible_spin: libadwaita::SpinRow,
    position_combo: libadwaita::ComboRow,
    follow_focus_switch: libadwaita::SwitchRow,
    preferences_group: libadwaita::PreferencesGroup,
    banner_group: libadwaita::PreferencesGroup,
}

impl NotificationsPage {
    /// Create a new notifications page
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
        preferences_group.set_title("Notification Settings");
        
        // Timeout spin row
        let adj = gtk4::Adjustment::new(5000.0, 1000.0, 60000.0, 100.0, 1000.0, 0.0);
        let timeout_spin = libadwaita::SpinRow::new(Some(&adj), 100.0, 0);
        timeout_spin.set_title("Timeout");
        timeout_spin.set_subtitle("How long notifications display before auto-dismissing");
        preferences_group.add(&timeout_spin);
        
        // Max visible spin row
        let adj = gtk4::Adjustment::new(5.0, 1.0, 20.0, 1.0, 1.0, 0.0);
        let max_visible_spin = libadwaita::SpinRow::new(Some(&adj), 1.0, 0);
        max_visible_spin.set_title("Maximum visible");
        max_visible_spin.set_subtitle("Maximum number of visible notifications");
        preferences_group.add(&max_visible_spin);
        
        // Position combo
        let position_model = gtk4::StringList::new(&[
            "Top Right", "Top Left", "Top Center",
            "Bottom Right", "Bottom Left", "Bottom Center",
            "Center"
        ]);
        let position_combo = libadwaita::ComboRow::new();
        position_combo.set_model(Some(&position_model));
        position_combo.set_title("Position");
        position_combo.set_subtitle("Where notifications appear on screen");
        position_combo.set_selected(0); // Default: TopRight
        preferences_group.add(&position_combo);
        
        // Follow focus switch
        let follow_focus_switch = libadwaita::SwitchRow::new();
        follow_focus_switch.set_title("Follow focus");
        follow_focus_switch.set_subtitle("Show notifications on focused monitor (if supported)");
        follow_focus_switch.set_active(false);
        preferences_group.add(&follow_focus_switch);
        
        prefs_page.add(&preferences_group);
        
        // Banner group for warnings (starts empty, will be populated by bind_detection)
        let banner_group = libadwaita::PreferencesGroup::new();
        prefs_page.add(&banner_group);
        
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);
        
        NotificationsPage {
            widget,
            timeout_spin,
            max_visible_spin,
            position_combo,
            follow_focus_switch,
            preferences_group,
            banner_group,
        }
    }
    
    /// Load notifications configuration into the page
    pub fn load_notifications(&self, config: &NotificationsConfig) {
        self.timeout_spin.set_value(config.timeout_ms as f64);
        self.max_visible_spin.set_value(config.max_visible as f64);
        
        let pos_idx = match config.position {
            NotifPosition::TopRight => 0,
            NotifPosition::TopLeft => 1,
            NotifPosition::TopCenter => 2,
            NotifPosition::BottomRight => 3,
            NotifPosition::BottomLeft => 4,
            NotifPosition::BottomCenter => 5,
            NotifPosition::Center => 6,
        };
        self.position_combo.set_selected(pos_idx);
        self.follow_focus_switch.set_active(config.follow_focus);
    }
    
    /// Bind feature detection and show warnings if needed
    pub fn bind_detection(&self, has_daemon: bool) {
        // Clear existing banner warnings
        while let Some(child) = self.banner_group.first_child() {
            self.banner_group.remove(&child);
        }
        
        // Disable preferences group if notification daemon is not found
        if !has_daemon {
            self.preferences_group.set_sensitive(false);
            
            let banner = libadwaita::ActionRow::new();
            banner.set_title("Warning: No notification daemon found");
            banner.set_subtitle("Install dunst, mako, or swaync to enable notifications");
            self.banner_group.add(&banner);
        } else {
            // Notification daemon available - enable controls and hide warnings
            self.preferences_group.set_sensitive(true);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for NotificationsPage {
    fn default() -> Self {
        Self::new()
    }
}
