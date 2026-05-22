/// Waybar configuration page
use crate::model::waybar::BarPosition;

/// Well-known waybar module names, grouped by typical position.
/// Left side
pub const MODULES_LEFT_OPTIONS: &[&str] = &[
    "sway/workspaces",
    "sway/mode",
    "sway/window",
    "sway/scratchpad",
];

/// Center
pub const MODULES_CENTER_OPTIONS: &[&str] = &[
    "clock",
    "sway/window",
];

/// Right side
pub const MODULES_RIGHT_OPTIONS: &[&str] = &[
    "battery",
    "network",
    "pulseaudio",
    "bluetooth",
    "cpu",
    "memory",
    "temperature",
    "tray",
];

// ── Pure helpers (no GTK) ──────────────────────────────────────────────────────

/// Index for a BarPosition in the ComboRow.
pub fn bar_position_index(p: BarPosition) -> u32 {
    match p {
        BarPosition::Top => 0,
        BarPosition::Bottom => 1,
        BarPosition::Left => 2,
        BarPosition::Right => 3,
    }
}

/// BarPosition from a ComboRow selected index.
pub fn bar_position_from_index(idx: u32) -> BarPosition {
    match idx {
        1 => BarPosition::Bottom,
        2 => BarPosition::Left,
        3 => BarPosition::Right,
        _ => BarPosition::Top,
    }
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
use crate::model::waybar::{WaybarConfig, WaybarModule};
#[cfg(feature = "gtk")]
use crate::state::AppState;

/// Waybar page - for configuring the status bar
#[cfg(feature = "gtk")]
pub struct WaybarPage {
    widget: gtk4::Box,
    enabled_switch: libadwaita::SwitchRow,
    position_combo: libadwaita::ComboRow,
    height_spin: libadwaita::SpinRow,
    preferences_group: libadwaita::PreferencesGroup,
    modules_group: libadwaita::PreferencesGroup,
    banner_group: libadwaita::PreferencesGroup,
    loading: Rc<Cell<bool>>,
    app_state: Rc<RefCell<AppState>>,
}

#[cfg(feature = "gtk")]
impl WaybarPage {
    /// Create a new waybar page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let prefs_page = libadwaita::PreferencesPage::new();
        
        // ── General group ─────────────────────────────────────────────────────
        let preferences_group = libadwaita::PreferencesGroup::new();
        preferences_group.set_title("Waybar Configuration");
        
        let enabled_switch = libadwaita::SwitchRow::new();
        enabled_switch.set_title("Enable waybar");
        enabled_switch.set_subtitle("Show the status bar on screen");
        enabled_switch.set_active(true);
        preferences_group.add(&enabled_switch);
        
        let position_model = gtk4::StringList::new(&["Top", "Bottom", "Left", "Right"]);
        let position_combo = libadwaita::ComboRow::new();
        position_combo.set_model(Some(&position_model));
        position_combo.set_title("Position");
        position_combo.set_subtitle("Position of the bar on screen");
        position_combo.set_selected(0);
        preferences_group.add(&position_combo);
        
        let adj = gtk4::Adjustment::new(30.0, 10.0, 200.0, 1.0, 5.0, 0.0);
        let height_spin = libadwaita::SpinRow::new(Some(&adj), 1.0, 0);
        height_spin.set_title("Height");
        height_spin.set_subtitle("Height of the bar in pixels");
        preferences_group.add(&height_spin);
        
        prefs_page.add(&preferences_group);

        // ── Module selection group ─────────────────────────────────────────────
        let modules_group = libadwaita::PreferencesGroup::new();
        modules_group.set_title("Modules");
        modules_group.set_description(Some("Select which modules appear in each bar section"));
        prefs_page.add(&modules_group);

        // ── Banner group ───────────────────────────────────────────────────────
        let banner_group = libadwaita::PreferencesGroup::new();
        prefs_page.add(&banner_group);
        
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        let loading = Rc::new(Cell::new(false));

        // ── enabled/position/height signals ────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            enabled_switch.connect_active_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().waybar.enabled = row.is_active();
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            position_combo.connect_selected_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().waybar.position =
                    bar_position_from_index(row.selected());
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let loading = Rc::clone(&loading);
            height_spin.connect_value_notify(move |row| {
                if loading.get() { return; }
                state.borrow_mut().settings_mut().waybar.height = row.value() as u32;
                state.borrow_mut().mark_dirty();
            });
        }

        WaybarPage {
            widget,
            enabled_switch,
            position_combo,
            height_spin,
            preferences_group,
            modules_group,
            banner_group,
            loading,
            app_state,
        }
    }
    
    /// Navigate to this page - load waybar config from app_state
    pub fn on_navigate(&self) {
        let config = self.app_state.borrow().settings().waybar.clone();
        self.load_waybar(&config);
        
        let has_waybar = crate::config::feature::has_waybar();
        self.bind_detection(has_waybar);
    }
    
    /// Load waybar configuration into the page (suppresses signal write-back)
    pub fn load_waybar(&self, config: &WaybarConfig) {
        self.loading.set(true);

        self.enabled_switch.set_active(config.enabled);
        self.position_combo.set_selected(bar_position_index(config.position));
        self.height_spin.set_value(config.height as f64);

        // Rebuild module checkboxes
        while let Some(child) = self.modules_group.first_child() {
            self.modules_group.remove(&child);
        }
        self.build_module_section("Left", MODULES_LEFT_OPTIONS, config, ModuleSection::Left);
        self.build_module_section("Center", MODULES_CENTER_OPTIONS, config, ModuleSection::Center);
        self.build_module_section("Right", MODULES_RIGHT_OPTIONS, config, ModuleSection::Right);

        self.loading.set(false);
    }

    fn build_module_section(
        &self,
        section_label: &str,
        options: &[&str],
        config: &WaybarConfig,
        section: ModuleSection,
    ) {
        let expander = libadwaita::ExpanderRow::new();
        expander.set_title(section_label);
        expander.set_expanded(true);

        for &module_name in options {
            let enabled = match section {
                ModuleSection::Left => config.modules_left.contains(&module_name.to_string()),
                ModuleSection::Center => config.modules_center.contains(&module_name.to_string()),
                ModuleSection::Right => config
                    .modules_right
                    .iter()
                    .any(|m| m.name == module_name && m.enabled),
            };

            let row = libadwaita::SwitchRow::new();
            row.set_title(module_name);
            row.set_active(enabled);

            let state = Rc::clone(&self.app_state);
            let name = module_name.to_string();
            row.connect_active_notify(move |r| {
                let on = r.is_active();
                let mut borrowed = state.borrow_mut();
                match section {
                    ModuleSection::Left => {
                        borrowed.settings_mut().waybar.modules_left.retain(|m| m != &name);
                        if on { borrowed.settings_mut().waybar.modules_left.push(name.clone()); }
                    }
                    ModuleSection::Center => {
                        borrowed.settings_mut().waybar.modules_center.retain(|m| m != &name);
                        if on { borrowed.settings_mut().waybar.modules_center.push(name.clone()); }
                    }
                    ModuleSection::Right => {
                        let modules = &mut borrowed.settings_mut().waybar.modules_right;
                        if let Some(m) = modules.iter_mut().find(|m| m.name == name) {
                            m.enabled = on;
                        } else if on {
                            modules.push(WaybarModule::new(&name, true));
                        }
                    }
                }
                borrowed.mark_dirty();
            });
            expander.add_row(&row);
        }

        self.modules_group.add(&expander);
    }

    /// Bind feature detection and show warnings if needed
    pub fn bind_detection(&self, has_waybar: bool) {
        while let Some(child) = self.banner_group.first_child() {
            self.banner_group.remove(&child);
        }
        
        if !has_waybar {
            self.preferences_group.set_sensitive(false);
            self.modules_group.set_sensitive(false);
            
            let banner = libadwaita::ActionRow::new();
            banner.set_title("Warning: Waybar is not installed");
            banner.set_subtitle("Install waybar to use this configuration");
            self.banner_group.add(&banner);
        } else {
            self.preferences_group.set_sensitive(true);
            self.modules_group.set_sensitive(true);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

#[cfg(feature = "gtk")]
#[derive(Clone, Copy)]
enum ModuleSection {
    Left,
    Center,
    Right,
}

#[cfg(feature = "gtk")]
impl Default for WaybarPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
