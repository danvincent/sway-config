/// Idle configuration page (screensaver, lock)
use gtk4::prelude::*;
use libadwaita::prelude::*;
use crate::model::idle::IdleConfig;
use crate::config::feature;

/// Idle page - for configuring idle behavior
pub struct IdlePage {
    widget: gtk4::Box,
    lock_timeout_spin: libadwaita::SpinRow,
    screen_off_timeout_spin: libadwaita::SpinRow,
    lock_command_entry: libadwaita::EntryRow,
    before_sleep_switch: libadwaita::SwitchRow,
    preferences_group: libadwaita::PreferencesGroup,
    banner_group: libadwaita::PreferencesGroup,
}

impl IdlePage {
    /// Create a new idle page
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
        preferences_group.set_title("Idle Settings");
        
        // Lock timeout spin row
        let lock_timeout_spin = libadwaita::SpinRow::new(
            gtk4::Adjustment::new(300.0, 0.0, 3600.0, 1.0, 60.0, 0.0),
            1.0,
            0,
        );
        lock_timeout_spin.set_title("Lock after");
        lock_timeout_spin.set_subtitle("Lock screen after this many seconds (0 = disabled)");
        lock_timeout_spin.set_has_subtitle(true);
        lock_timeout_spin.set_suffix_expression(Some(
            gtk4::Expression::constant("seconds")
        ));
        preferences_group.add(&lock_timeout_spin);
        
        // Screen off timeout spin row
        let screen_off_timeout_spin = libadwaita::SpinRow::new(
            gtk4::Adjustment::new(0.0, 0.0, 3600.0, 1.0, 60.0, 0.0),
            1.0,
            0,
        );
        screen_off_timeout_spin.set_title("Screen off after");
        screen_off_timeout_spin.set_subtitle("Turn screen off after this many seconds (0 = disabled)");
        screen_off_timeout_spin.set_has_subtitle(true);
        screen_off_timeout_spin.set_suffix_expression(Some(
            gtk4::Expression::constant("seconds")
        ));
        preferences_group.add(&screen_off_timeout_spin);
        
        // Lock command entry
        let lock_command_entry = libadwaita::EntryRow::new();
        lock_command_entry.set_title("Lock command");
        lock_command_entry.set_text("swaylock");
        preferences_group.add(&lock_command_entry);
        
        // Before sleep switch
        let before_sleep_switch = libadwaita::SwitchRow::new();
        before_sleep_switch.set_title("Lock before sleep");
        before_sleep_switch.set_subtitle("Lock screen when system goes to sleep");
        before_sleep_switch.set_has_subtitle(true);
        before_sleep_switch.set_active(true);
        preferences_group.add(&before_sleep_switch);
        
        prefs_page.add(&preferences_group);
        
        // Banner group for warnings (starts empty, will be populated by bind_detection)
        let banner_group = libadwaita::PreferencesGroup::new();
        prefs_page.add(&banner_group);
        
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);
        
        IdlePage {
            widget,
            lock_timeout_spin,
            screen_off_timeout_spin,
            lock_command_entry,
            before_sleep_switch,
            preferences_group,
            banner_group,
        }
    }
    
    /// Load idle configuration into the page
    pub fn load_idle(&self, config: &IdleConfig) {
        self.lock_timeout_spin.set_value(config.lock_timeout as f64);
        self.screen_off_timeout_spin.set_value(config.screen_off_timeout as f64);
        self.lock_command_entry.set_text(&config.lock_command);
        self.before_sleep_switch.set_active(config.before_sleep);
    }
    
    /// Bind feature detection and show warnings if needed
    pub fn bind_detection(&self, has_swayidle: bool, has_locker: bool) {
        // Clear existing banner warnings
        while let Some(child) = self.banner_group.first_child() {
            self.banner_group.remove(&child);
        }
        
        // Disable preferences group if critical features are missing
        if !has_swayidle || !has_locker {
            self.preferences_group.set_sensitive(false);
            
            if !has_swayidle {
                let banner = libadwaita::ActionRow::new();
                banner.set_title("Warning: swayidle not found");
                banner.set_subtitle("Install swayidle to use idle locking");
                self.banner_group.add(&banner);
            }
            if !has_locker {
                let banner = libadwaita::ActionRow::new();
                banner.set_title("Warning: No screen locker found");
                banner.set_subtitle("Install swaylock or waylock");
                self.banner_group.add(&banner);
            }
        } else {
            // All features available - enable controls and hide warnings
            self.preferences_group.set_sensitive(true);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for IdlePage {
    fn default() -> Self {
        Self::new()
    }
}
