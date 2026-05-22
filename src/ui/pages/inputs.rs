/// Inputs configuration page (keyboard, mouse)
#[cfg(feature = "gtk")]
use gtk4::prelude::*;
#[cfg(feature = "gtk")]
use std::cell::RefCell;
#[cfg(feature = "gtk")]
use std::rc::Rc;

#[cfg(feature = "gtk")]
use libadwaita::prelude::*;

use crate::model::input::AccelProfile;

#[cfg(feature = "gtk")]
use crate::model::input::{KeyboardConfig, TouchpadConfig};

#[cfg(feature = "gtk")]
use crate::state::AppState;

// ─── Pure helpers (no GTK — testable without a display) ───────────────────────

/// Labels for the acceleration-profile ComboRow, in index order.
pub const ACCEL_PROFILES: [AccelProfile; 2] = [AccelProfile::Adaptive, AccelProfile::Flat];

/// Human-readable label for an AccelProfile value.
pub fn accel_profile_label(p: AccelProfile) -> &'static str {
    match p {
        AccelProfile::Adaptive => "Adaptive",
        AccelProfile::Flat => "Flat",
    }
}

/// Index of an AccelProfile in ACCEL_PROFILES.
pub fn accel_profile_index(p: AccelProfile) -> u32 {
    ACCEL_PROFILES.iter().position(|&x| x == p).unwrap_or(0) as u32
}

/// AccelProfile from a ComboRow selected index.
pub fn accel_profile_from_index(idx: u32) -> AccelProfile {
    ACCEL_PROFILES
        .get(idx as usize)
        .copied()
        .unwrap_or(AccelProfile::Adaptive)
}

// ─── GTK page ─────────────────────────────────────────────────────────────────

/// Inputs page - for configuring input devices
#[cfg(feature = "gtk")]
pub struct InputsPage {
    widget: gtk4::Box,
    groups_box: gtk4::Box,
    app_state: Rc<RefCell<AppState>>,
}

#[cfg(feature = "gtk")]
impl InputsPage {
    /// Create a new inputs page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);

        let groups_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        groups_box.set_margin_start(12);
        groups_box.set_margin_end(12);
        groups_box.set_margin_top(12);
        groups_box.set_margin_bottom(12);

        clamp.set_child(Some(&groups_box));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        InputsPage {
            widget,
            groups_box,
            app_state,
        }
    }

    /// Detect and load inputs into the page
    pub fn on_navigate(&self) {
        let need_keyboards = self.app_state.borrow().should_refresh_keyboards();
        let need_touchpads = self.app_state.borrow().should_refresh_touchpads();

        if need_keyboards || need_touchpads {
            let all_inputs = crate::config::detect::detect_inputs();

            if need_keyboards {
                let keyboard_inputs = crate::config::detect::detect_keyboards(&all_inputs);
                let keyboards: Vec<KeyboardConfig> = keyboard_inputs
                    .iter()
                    .map(|i| KeyboardConfig::from_sway(i))
                    .collect();
                self.app_state.borrow_mut().refresh_keyboards(keyboards);
            }

            if need_touchpads {
                let touchpad_inputs = crate::config::detect::detect_touchpads(&all_inputs);
                let touchpads: Vec<TouchpadConfig> = touchpad_inputs
                    .iter()
                    .map(|i| TouchpadConfig::from_sway(i))
                    .collect();
                self.app_state.borrow_mut().refresh_touchpads(touchpads);
            }
        }

        let keyboards = self.app_state.borrow().settings().keyboards.clone();
        let touchpads = self.app_state.borrow().settings().touchpads.clone();
        self.load_inputs(&keyboards, &touchpads);
    }

    /// Rebuild all groups from current settings
    pub fn load_inputs(&self, keyboards: &[KeyboardConfig], touchpads: &[TouchpadConfig]) {
        while let Some(child) = self.groups_box.first_child() {
            self.groups_box.remove(&child);
        }

        if keyboards.is_empty() && touchpads.is_empty() {
            let label = gtk4::Label::new(Some("No input devices detected.\nConnect a keyboard or touchpad and navigate away then back."));
            label.set_halign(gtk4::Align::Center);
            label.add_css_class("dim-label");
            label.set_wrap(true);
            label.set_justify(gtk4::Justification::Center);
            label.set_margin_top(48);
            self.groups_box.append(&label);
            return;
        }

        for (idx, kb) in keyboards.iter().enumerate() {
            let group = self.build_keyboard_group(kb, idx);
            self.groups_box.append(&group);
        }

        for (idx, tp) in touchpads.iter().enumerate() {
            let group = self.build_touchpad_group(tp, idx);
            self.groups_box.append(&group);
        }
    }

    fn build_keyboard_group(
        &self,
        kb: &KeyboardConfig,
        idx: usize,
    ) -> libadwaita::PreferencesGroup {
        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Keyboard");

        let expander = libadwaita::ExpanderRow::new();
        expander.set_title(&kb.identifier);
        expander.set_subtitle("Keyboard device");
        expander.set_expanded(true);

        // ── Layout ──
        let layout_row = libadwaita::EntryRow::new();
        layout_row.set_title("XKB Layout");
        layout_row.set_text(&kb.xkb_layout);
        {
            let state = Rc::clone(&self.app_state);
            layout_row.connect_changed(move |entry| {
                let text = entry.text().to_string();
                if let Some(kb) = state.borrow_mut().settings_mut().keyboards.get_mut(idx) {
                    kb.xkb_layout = text;
                }
                state.borrow_mut().mark_keyboards_dirty();
            });
        }
        expander.add_row(&layout_row);

        // ── Variant ──
        let variant_row = libadwaita::EntryRow::new();
        variant_row.set_title("XKB Variant");
        variant_row.set_text(&kb.xkb_variant);
        {
            let state = Rc::clone(&self.app_state);
            variant_row.connect_changed(move |entry| {
                let text = entry.text().to_string();
                if let Some(kb) = state.borrow_mut().settings_mut().keyboards.get_mut(idx) {
                    kb.xkb_variant = text;
                }
                state.borrow_mut().mark_keyboards_dirty();
            });
        }
        expander.add_row(&variant_row);

        // ── Options ──
        let options_row = libadwaita::EntryRow::new();
        options_row.set_title("XKB Options");
        options_row.set_text(&kb.xkb_options);
        {
            let state = Rc::clone(&self.app_state);
            options_row.connect_changed(move |entry| {
                let text = entry.text().to_string();
                if let Some(kb) = state.borrow_mut().settings_mut().keyboards.get_mut(idx) {
                    kb.xkb_options = text;
                }
                state.borrow_mut().mark_keyboards_dirty();
            });
        }
        expander.add_row(&options_row);

        // ── Repeat delay ──
        let delay_row = libadwaita::SpinRow::with_range(100.0, 2000.0, 50.0);
        delay_row.set_title("Repeat Delay (ms)");
        delay_row.set_value(kb.repeat_delay as f64);
        {
            let state = Rc::clone(&self.app_state);
            delay_row.connect_value_notify(move |row| {
                let val = row.value() as i32;
                if let Some(kb) = state.borrow_mut().settings_mut().keyboards.get_mut(idx) {
                    kb.repeat_delay = val;
                }
                state.borrow_mut().mark_keyboards_dirty();
            });
        }
        expander.add_row(&delay_row);

        // ── Repeat rate ──
        let rate_row = libadwaita::SpinRow::with_range(1.0, 100.0, 1.0);
        rate_row.set_title("Repeat Rate (keys/s)");
        rate_row.set_value(kb.repeat_rate as f64);
        {
            let state = Rc::clone(&self.app_state);
            rate_row.connect_value_notify(move |row| {
                let val = row.value() as i32;
                if let Some(kb) = state.borrow_mut().settings_mut().keyboards.get_mut(idx) {
                    kb.repeat_rate = val;
                }
                state.borrow_mut().mark_keyboards_dirty();
            });
        }
        expander.add_row(&rate_row);

        group.add(&expander);
        group
    }

    fn build_touchpad_group(
        &self,
        tp: &TouchpadConfig,
        idx: usize,
    ) -> libadwaita::PreferencesGroup {
        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Touchpad");

        let expander = libadwaita::ExpanderRow::new();
        expander.set_title(&tp.identifier);
        expander.set_subtitle("Touchpad device");
        expander.set_expanded(true);

        macro_rules! add_switch {
            ($title:expr, $val:expr, $field:ident) => {{
                let row = libadwaita::SwitchRow::new();
                row.set_title($title);
                row.set_active($val);
                let state = Rc::clone(&self.app_state);
                row.connect_active_notify(move |r| {
                    let v = r.is_active();
                    if let Some(tp) = state.borrow_mut().settings_mut().touchpads.get_mut(idx) {
                        tp.$field = v;
                    }
                    state.borrow_mut().mark_touchpads_dirty();
                });
                row
            }};
        }

        let tap_row = add_switch!("Tap to Click", tp.tap_to_click, tap_to_click);
        expander.add_row(&tap_row);

        let scroll_row = add_switch!("Natural Scroll", tp.natural_scroll, natural_scroll);
        expander.add_row(&scroll_row);

        let dwt_row = add_switch!("Disable While Typing", tp.dwt, dwt);
        expander.add_row(&dwt_row);

        let left_row = add_switch!("Left-Handed Mode", tp.left_handed, left_handed);
        expander.add_row(&left_row);

        let middle_row = add_switch!(
            "Middle Button Emulation",
            tp.middle_emulation,
            middle_emulation
        );
        expander.add_row(&middle_row);

        // ── Accel speed ──
        let accel_speed_row = libadwaita::SpinRow::with_range(-1.0, 1.0, 0.1);
        accel_speed_row.set_title("Acceleration Speed");
        accel_speed_row.set_digits(1);
        accel_speed_row.set_value(tp.accel_speed);
        {
            let state = Rc::clone(&self.app_state);
            accel_speed_row.connect_value_notify(move |row| {
                let val = row.value();
                if let Some(tp) = state.borrow_mut().settings_mut().touchpads.get_mut(idx) {
                    tp.accel_speed = val;
                }
                state.borrow_mut().mark_touchpads_dirty();
            });
        }
        expander.add_row(&accel_speed_row);

        // ── Accel profile ──
        let profile_strings: gtk4::StringList =
            gtk4::StringList::new(&ACCEL_PROFILES.map(accel_profile_label));
        let profile_row = libadwaita::ComboRow::new();
        profile_row.set_title("Acceleration Profile");
        profile_row.set_model(Some(&profile_strings));
        profile_row.set_selected(accel_profile_index(tp.accel_profile));
        {
            let state = Rc::clone(&self.app_state);
            profile_row.connect_selected_notify(move |row| {
                let profile = accel_profile_from_index(row.selected());
                if let Some(tp) = state.borrow_mut().settings_mut().touchpads.get_mut(idx) {
                    tp.accel_profile = profile;
                }
                state.borrow_mut().mark_touchpads_dirty();
            });
        }
        expander.add_row(&profile_row);

        group.add(&expander);
        group
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

#[cfg(feature = "gtk")]
impl Default for InputsPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}

#[cfg(not(feature = "gtk"))]
pub struct InputsPage;
