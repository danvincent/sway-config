/// Notifications configuration page
use crate::model::notifications::NotifPosition;

// ── Pure helpers (no GTK) ──────────────────────────────────────────────────────

/// Labels for the notifications position ComboRow, in index order.
pub const NOTIF_POSITIONS: [NotifPosition; 7] = [
    NotifPosition::TopRight,
    NotifPosition::TopLeft,
    NotifPosition::TopCenter,
    NotifPosition::BottomRight,
    NotifPosition::BottomLeft,
    NotifPosition::BottomCenter,
    NotifPosition::Center,
];

/// Human-readable label for a NotifPosition.
pub fn notif_position_label(p: NotifPosition) -> &'static str {
    match p {
        NotifPosition::TopRight => "Top Right",
        NotifPosition::TopLeft => "Top Left",
        NotifPosition::TopCenter => "Top Center",
        NotifPosition::BottomRight => "Bottom Right",
        NotifPosition::BottomLeft => "Bottom Left",
        NotifPosition::BottomCenter => "Bottom Center",
        NotifPosition::Center => "Center",
    }
}

/// Index of a NotifPosition in NOTIF_POSITIONS.
pub fn notif_position_index(p: NotifPosition) -> u32 {
    NOTIF_POSITIONS.iter().position(|&x| x == p).unwrap_or(0) as u32
}

/// NotifPosition from a ComboRow selected index.
pub fn notif_position_from_index(idx: u32) -> NotifPosition {
    NOTIF_POSITIONS.get(idx as usize).copied().unwrap_or(NotifPosition::TopRight)
}

// ── GTK page ──────────────────────────────────────────────────────────────────

#[cfg(feature = "gtk")]
use gtk4::prelude::*;
#[cfg(feature = "gtk")]
use libadwaita::prelude::*;
#[cfg(feature = "gtk")]
use std::cell::{Cell, RefCell};
#[cfg(feature = "gtk")]
use std::rc::Rc;
#[cfg(feature = "gtk")]
use crate::model::notifications::NotificationsConfig;
#[cfg(feature = "gtk")]
use crate::state::AppState;

/// Notifications page - for configuring notifications
#[cfg(feature = "gtk")]
pub struct NotificationsPage {
    widget: gtk4::Box,
    timeout_spin: libadwaita::SpinRow,
    max_visible_spin: libadwaita::SpinRow,
    position_combo: libadwaita::ComboRow,
    follow_focus_switch: libadwaita::SwitchRow,
    preferences_group: libadwaita::PreferencesGroup,
    banner_group: libadwaita::PreferencesGroup,
    loading: Rc<Cell<bool>>,
    app_state: Rc<RefCell<AppState>>,
}

#[cfg(feature = "gtk")]
impl NotificationsPage {
    /// Create a new notifications page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let prefs_page = libadwaita::PreferencesPage::new();
        
        let preferences_group = libadwaita::PreferencesGroup::new();
        preferences_group.set_title("Notification Settings");
        
        let adj = gtk4::Adjustment::new(5000.0, 1000.0, 60000.0, 100.0, 1000.0, 0.0);
        let timeout_spin = libadwaita::SpinRow::new(Some(&adj), 100.0, 0);
        timeout_spin.set_title("Timeout (ms)");
        timeout_spin.set_subtitle("How long notifications display before auto-dismissing");
        preferences_group.add(&timeout_spin);
        
        let adj = gtk4::Adjustment::new(5.0, 1.0, 20.0, 1.0, 1.0, 0.0);
        let max_visible_spin = libadwaita::SpinRow::new(Some(&adj), 1.0, 0);
        max_visible_spin.set_title("Maximum visible");
        max_visible_spin.set_subtitle("Maximum number of visible notifications");
        preferences_group.add(&max_visible_spin);
        
        let position_labels: Vec<&str> = NOTIF_POSITIONS.iter().map(|&p| notif_position_label(p)).collect();
        let position_model = gtk4::StringList::new(&position_labels);
        let position_combo = libadwaita::ComboRow::new();
        position_combo.set_model(Some(&position_model));
        position_combo.set_title("Position");
        position_combo.set_subtitle("Where notifications appear on screen");
        position_combo.set_selected(0);
        preferences_group.add(&position_combo);
        
        let follow_focus_switch = libadwaita::SwitchRow::new();
        follow_focus_switch.set_title("Follow focus");
        follow_focus_switch.set_subtitle("Show notifications on focused monitor (if supported)");
        follow_focus_switch.set_active(false);
        preferences_group.add(&follow_focus_switch);
        
        prefs_page.add(&preferences_group);
        
        let banner_group = libadwaita::PreferencesGroup::new();
        prefs_page.add(&banner_group);
        
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        let loading = Rc::new(Cell::new(false));

        // ── Signal connections ──────────────────────────────────────────────────

        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            timeout_spin.connect_value_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().notifications.timeout_ms = row.value() as u32;
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            max_visible_spin.connect_value_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().notifications.max_visible = row.value() as u32;
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            position_combo.connect_selected_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().notifications.position =
                    notif_position_from_index(row.selected());
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            follow_focus_switch.connect_active_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().notifications.follow_focus = row.is_active();
                state.borrow_mut().mark_dirty();
            });
        }

        NotificationsPage {
            widget,
            timeout_spin,
            max_visible_spin,
            position_combo,
            follow_focus_switch,
            preferences_group,
            banner_group,
            loading,
            app_state,
        }
    }
    
    /// Navigate to this page - load notifications config from app_state
    pub fn on_navigate(&self) {
        let config = self.app_state.borrow().settings().notifications.clone();
        self.load_notifications(&config);
        
        let has_daemon = crate::config::feature::has_notification_daemon();
        self.bind_detection(has_daemon);
    }
    
    /// Load notifications configuration into the page (suppresses signal write-back)
    pub fn load_notifications(&self, config: &NotificationsConfig) {
        self.loading.set(true);
        self.timeout_spin.set_value(config.timeout_ms as f64);
        self.max_visible_spin.set_value(config.max_visible as f64);
        self.position_combo.set_selected(notif_position_index(config.position));
        self.follow_focus_switch.set_active(config.follow_focus);
        self.loading.set(false);
    }
    
    /// Bind feature detection and show warnings if needed
    pub fn bind_detection(&self, has_daemon: bool) {
        while let Some(child) = self.banner_group.first_child() {
            self.banner_group.remove(&child);
        }
        
        if !has_daemon {
            self.preferences_group.set_sensitive(false);
            
            let banner = libadwaita::ActionRow::new();
            banner.set_title("Warning: No notification daemon found");
            banner.set_subtitle("Install dunst, mako, or swaync to enable notifications");
            self.banner_group.add(&banner);
        } else {
            self.preferences_group.set_sensitive(true);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

#[cfg(feature = "gtk")]
impl Default for NotificationsPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
