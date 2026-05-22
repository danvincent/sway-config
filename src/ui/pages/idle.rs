/// Idle configuration page (screensaver, lock)
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use crate::model::idle::IdleConfig;
use crate::state::AppState;

/// Idle page - for configuring idle behavior
pub struct IdlePage {
    widget: gtk4::Box,
    lock_timeout_spin: libadwaita::SpinRow,
    screen_off_timeout_spin: libadwaita::SpinRow,
    lock_command_entry: libadwaita::EntryRow,
    before_sleep_switch: libadwaita::SwitchRow,
    preferences_group: libadwaita::PreferencesGroup,
    banner_group: libadwaita::PreferencesGroup,
    swayidle_banner: libadwaita::ActionRow,
    locker_banner: libadwaita::ActionRow,
    loading: Rc<Cell<bool>>,
    app_state: Rc<RefCell<AppState>>,
}

impl IdlePage {
    /// Create a new idle page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
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
        let adj = gtk4::Adjustment::new(300.0, 0.0, 3600.0, 1.0, 60.0, 0.0);
        let lock_timeout_spin = libadwaita::SpinRow::new(Some(&adj), 1.0, 0);
        lock_timeout_spin.set_title("Lock after");
        lock_timeout_spin.set_subtitle("Lock screen after this many seconds (0 = disabled)");
        preferences_group.add(&lock_timeout_spin);
        
        // Screen off timeout spin row
        let adj = gtk4::Adjustment::new(0.0, 0.0, 3600.0, 1.0, 60.0, 0.0);
        let screen_off_timeout_spin = libadwaita::SpinRow::new(Some(&adj), 1.0, 0);
        screen_off_timeout_spin.set_title("Screen off after");
        screen_off_timeout_spin.set_subtitle("Turn screen off after this many seconds (0 = disabled)");
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
        before_sleep_switch.set_active(true);
        preferences_group.add(&before_sleep_switch);
        
        prefs_page.add(&preferences_group);
        
        // Banner group: rows created once, shown/hidden via set_visible() — never rebuilt.
        // (Rebuilding by calling first_child()+remove() on PreferencesGroup is not supported;
        //  first_child() returns an internal GtkBox, not your rows.)
        let banner_group = libadwaita::PreferencesGroup::new();
        let swayidle_banner = libadwaita::ActionRow::new();
        swayidle_banner.set_title("Warning: swayidle not found");
        swayidle_banner.set_subtitle("Install swayidle to use idle locking");
        let locker_banner = libadwaita::ActionRow::new();
        locker_banner.set_title("Warning: No screen locker found");
        locker_banner.set_subtitle("Install swaylock or waylock");
        banner_group.add(&swayidle_banner);
        banner_group.add(&locker_banner);
        swayidle_banner.set_visible(false);
        locker_banner.set_visible(false);
        banner_group.set_visible(false);
        prefs_page.add(&banner_group);
        
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        let loading = Rc::new(Cell::new(false));

        // ── Signal connections ──────────────────────────────────────────────────

        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            lock_timeout_spin.connect_value_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().idle.lock_timeout = row.value() as u32;
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            screen_off_timeout_spin.connect_value_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().idle.screen_off_timeout = row.value() as u32;
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            lock_command_entry.connect_changed(move |entry| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().idle.lock_command = entry.text().to_string();
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            before_sleep_switch.connect_active_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().idle.before_sleep = row.is_active();
                state.borrow_mut().mark_dirty();
            });
        }

        IdlePage {
            widget,
            lock_timeout_spin,
            screen_off_timeout_spin,
            lock_command_entry,
            before_sleep_switch,
            preferences_group,
            banner_group,
            swayidle_banner,
            locker_banner,
            loading,
            app_state,
        }
    }
    
    /// Navigate to this page - load idle config from app_state
    pub fn on_navigate(&self) {
        let config = self.app_state.borrow().settings().idle.clone();
        self.load_idle(&config);
        
        let has_swayidle = crate::config::feature::has_swayidle();
        let has_locker = crate::config::feature::has_screen_locker();
        self.bind_detection(has_swayidle, has_locker);
    }
    
    /// Load idle configuration into the page (suppresses signal write-back)
    pub fn load_idle(&self, config: &IdleConfig) {
        self.loading.set(true);
        self.lock_timeout_spin.set_value(config.lock_timeout as f64);
        self.screen_off_timeout_spin.set_value(config.screen_off_timeout as f64);
        self.lock_command_entry.set_text(&config.lock_command);
        self.before_sleep_switch.set_active(config.before_sleep);
        self.loading.set(false);
    }
    
    /// Bind feature detection and show warnings if needed
    pub fn bind_detection(&self, has_swayidle: bool, has_locker: bool) {
        self.swayidle_banner.set_visible(!has_swayidle);
        self.locker_banner.set_visible(!has_locker);
        let show_banner = !has_swayidle || !has_locker;
        self.banner_group.set_visible(show_banner);
        self.preferences_group.set_sensitive(has_swayidle && has_locker);
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for IdlePage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
