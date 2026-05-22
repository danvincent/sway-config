/// Outputs configuration page (displays/monitors)
#[cfg(feature = "gtk")]
use gtk4::prelude::*;
#[cfg(feature = "gtk")]
use libadwaita::prelude::*;
#[cfg(feature = "gtk")]
use std::cell::RefCell;
#[cfg(feature = "gtk")]
use std::rc::Rc;

#[cfg(feature = "gtk")]
use crate::config::detect::SwayOutput;
use crate::model::output::Transform;
#[cfg(feature = "gtk")]
use crate::model::output::{OutputConfig, Resolution};
#[cfg(feature = "gtk")]
use crate::state::AppState;

/// Format a display mode as "WxH @ Hz" for use in ComboRow items.
/// `refresh_mhz` is the raw sway refresh value (millihertz, e.g. 60000 = 60Hz).
pub fn format_mode(width: i32, height: i32, refresh_mhz: i32) -> String {
    format!("{}×{} @ {}Hz", width, height, refresh_mhz / 1000)
}

/// Human-readable label for each Transform variant (shown in the ComboRow).
pub fn transform_label(t: Transform) -> &'static str {
    match t {
        Transform::Normal => "Normal",
        Transform::Rotate90 => "90°",
        Transform::Rotate180 => "180°",
        Transform::Rotate270 => "270°",
        Transform::Flipped => "Flipped",
        Transform::Flipped90 => "Flipped 90°",
        Transform::Flipped180 => "Flipped 180°",
        Transform::Flipped270 => "Flipped 270°",
    }
}

/// Index in the TRANSFORMS array for a given Transform variant.
pub fn transform_index(t: Transform) -> u32 {
    TRANSFORMS.iter().position(|&x| x == t).unwrap_or(0) as u32
}

/// Transform variant from a ComboRow selection index.
pub fn transform_from_index(idx: u32) -> Transform {
    TRANSFORMS
        .get(idx as usize)
        .copied()
        .unwrap_or(Transform::Normal)
}

pub const TRANSFORMS: &[Transform] = &[
    Transform::Normal,
    Transform::Rotate90,
    Transform::Rotate180,
    Transform::Rotate270,
    Transform::Flipped,
    Transform::Flipped90,
    Transform::Flipped180,
    Transform::Flipped270,
];

/// Outputs page - for configuring monitor layout
#[cfg(feature = "gtk")]
pub struct OutputsPage {
    widget: gtk4::Box,
    groups_box: gtk4::Box,
    app_state: Rc<RefCell<AppState>>,
    /// Raw detected outputs from sway — kept for available-modes data on the ComboRow.
    detected_outputs: RefCell<Vec<SwayOutput>>,
}

#[cfg(feature = "gtk")]
impl OutputsPage {
    /// Create a new outputs page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);

        let groups_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        groups_box.set_margin_start(12);
        groups_box.set_margin_end(12);
        groups_box.set_margin_top(12);
        groups_box.set_margin_bottom(12);

        let scrolled = gtk4::ScrolledWindow::new();
        clamp.set_child(Some(&groups_box));
        scrolled.set_child(Some(&clamp));
        scrolled.set_vexpand(true);
        widget.append(&scrolled);

        OutputsPage {
            widget,
            groups_box,
            app_state,
            detected_outputs: RefCell::new(vec![]),
        }
    }

    /// Called when the user navigates to this page.
    /// Detects outputs (if not dirty) then rebuilds the UI.
    pub fn on_navigate(&self) {
        if self.app_state.borrow().should_refresh_outputs() {
            let detected = crate::config::detect::detect_outputs();
            *self.detected_outputs.borrow_mut() = detected.clone();
            let configs: Vec<OutputConfig> = detected
                .iter()
                .map(|o| OutputConfig::from_sway(o))
                .collect();
            self.app_state.borrow_mut().refresh_outputs(configs);
        }

        let settings = self.app_state.borrow().settings().outputs.clone();
        let detected = self.detected_outputs.borrow().clone();
        self.load_outputs(&settings, &detected);
    }

    /// Rebuild the outputs list from the given settings and (optionally) raw detected data.
    /// `detected` provides available modes for the resolution ComboRow.
    pub fn load_outputs(&self, outputs: &[OutputConfig], detected: &[SwayOutput]) {
        while let Some(child) = self.groups_box.first_child() {
            self.groups_box.remove(&child);
        }

        if outputs.is_empty() {
            let label = gtk4::Label::new(Some(
                "No displays detected. Connect a display and navigate here.",
            ));
            label.add_css_class("dim-label");
            label.set_wrap(true);
            self.groups_box.append(&label);
            return;
        }

        for (idx, output) in outputs.iter().enumerate() {
            // Match detected SwayOutput by name, not index, to handle ordering differences
            let sway_out = detected.iter().find(|d| d.name == output.name);
            let group = self.build_output_group(idx, output, sway_out);
            self.groups_box.append(&group);
        }
    }

    /// Build the PreferencesGroup (with ExpanderRow) for a single output.
    fn build_output_group(
        &self,
        idx: usize,
        output: &OutputConfig,
        sway_out: Option<&SwayOutput>,
    ) -> libadwaita::PreferencesGroup {
        let group = libadwaita::PreferencesGroup::new();
        group.set_title(&output.name);
        if let Some(detected) = sway_out {
            if !detected.make.is_empty() && detected.make != "Unknown" {
                group.set_description(Some(&format!("{} {}", detected.make, detected.model)));
            }
        }

        let expander = libadwaita::ExpanderRow::new();
        expander.set_title("Configure");
        if let Some(res) = &output.resolution {
            let hz = output.refresh_rate.map(|r| r / 1000).unwrap_or(0);
            expander.set_subtitle(&format!("{}×{} @ {}Hz", res.width, res.height, hz));
        } else {
            expander.set_subtitle("Auto");
        }

        // ── Enabled toggle ───────────────────────────────────────────
        let switch_row = libadwaita::SwitchRow::new();
        switch_row.set_title("Enabled");
        switch_row.set_active(output.enabled);
        {
            let app = self.app_state.clone();
            switch_row.connect_active_notify(move |row| {
                let mut state = app.borrow_mut();
                if let Some(out) = state.settings_mut().outputs.get_mut(idx) {
                    out.enabled = row.is_active();
                }
                state.mark_outputs_dirty();
            });
        }
        expander.add_row(&switch_row);

        // ── Resolution ComboRow (only when detected modes are available) ──
        let modes_data: Vec<(i32, i32, i32)> = sway_out
            .map(|o| {
                o.modes
                    .iter()
                    .map(|m| (m.width, m.height, m.refresh))
                    .collect()
            })
            .unwrap_or_default();

        if !modes_data.is_empty() {
            // "Auto" is index 0; detected modes start at index 1.
            let mut mode_strings: Vec<String> = vec!["Auto".to_string()];
            mode_strings.extend(modes_data.iter().map(|&(w, h, r)| format_mode(w, h, r)));
            let model =
                gtk4::StringList::new(&mode_strings.iter().map(|s| s.as_str()).collect::<Vec<_>>());

            let res_row = libadwaita::ComboRow::new();
            res_row.set_title("Resolution");
            res_row.set_model(Some(&model));

            // Pre-select current resolution (or Auto if none set)
            let selected_idx = output
                .resolution
                .as_ref()
                .and_then(|res| {
                    let current =
                        format_mode(res.width, res.height, output.refresh_rate.unwrap_or(0));
                    mode_strings.iter().position(|m| m == &current)
                })
                .unwrap_or(0); // default to Auto
            res_row.set_selected(selected_idx as u32);

            {
                let app = self.app_state.clone();
                let modes_clone = modes_data.clone();
                res_row.connect_selected_notify(move |row| {
                    let sel = row.selected() as usize;
                    let mut state = app.borrow_mut();
                    if let Some(out) = state.settings_mut().outputs.get_mut(idx) {
                        if sel == 0 {
                            // Auto — let sway pick
                            out.resolution = None;
                            out.refresh_rate = None;
                        } else if let Some(&(w, h, r)) = modes_clone.get(sel - 1) {
                            out.resolution = Some(Resolution {
                                width: w,
                                height: h,
                            });
                            out.refresh_rate = Some(r);
                        }
                    }
                    state.mark_outputs_dirty();
                });
            }
            expander.add_row(&res_row);
        }

        // ── Scale SpinRow ────────────────────────────────────────────
        let scale_row = libadwaita::SpinRow::with_range(0.25, 4.0, 0.25);
        scale_row.set_title("Scale");
        scale_row.set_digits(2);
        scale_row.set_value(output.scale);
        {
            let app = self.app_state.clone();
            scale_row.connect_value_notify(move |row| {
                let mut state = app.borrow_mut();
                if let Some(out) = state.settings_mut().outputs.get_mut(idx) {
                    out.scale = row.value();
                }
                state.mark_outputs_dirty();
            });
        }
        expander.add_row(&scale_row);

        // ── Transform ComboRow ───────────────────────────────────────
        let t_labels: Vec<&str> = TRANSFORMS.iter().map(|t| transform_label(*t)).collect();
        let t_model = gtk4::StringList::new(&t_labels);
        let transform_row = libadwaita::ComboRow::new();
        transform_row.set_title("Transform");
        transform_row.set_model(Some(&t_model));
        transform_row.set_selected(transform_index(output.transform));
        {
            let app = self.app_state.clone();
            transform_row.connect_selected_notify(move |row| {
                let mut state = app.borrow_mut();
                if let Some(out) = state.settings_mut().outputs.get_mut(idx) {
                    out.transform = transform_from_index(row.selected());
                }
                state.mark_outputs_dirty();
            });
        }
        expander.add_row(&transform_row);

        // ── Position X SpinRow ───────────────────────────────────────
        let pos_x_row = libadwaita::SpinRow::with_range(-8192.0, 8192.0, 1.0);
        pos_x_row.set_title("Position X");
        pos_x_row.set_value(output.position.x as f64);
        {
            let app = self.app_state.clone();
            pos_x_row.connect_value_notify(move |row| {
                let mut state = app.borrow_mut();
                if let Some(out) = state.settings_mut().outputs.get_mut(idx) {
                    out.position.x = row.value() as i32;
                }
                state.mark_outputs_dirty();
            });
        }
        expander.add_row(&pos_x_row);

        // ── Position Y SpinRow ───────────────────────────────────────
        let pos_y_row = libadwaita::SpinRow::with_range(-8192.0, 8192.0, 1.0);
        pos_y_row.set_title("Position Y");
        pos_y_row.set_value(output.position.y as f64);
        {
            let app = self.app_state.clone();
            pos_y_row.connect_value_notify(move |row| {
                let mut state = app.borrow_mut();
                if let Some(out) = state.settings_mut().outputs.get_mut(idx) {
                    out.position.y = row.value() as i32;
                }
                state.mark_outputs_dirty();
            });
        }
        expander.add_row(&pos_y_row);

        group.add(&expander);
        group
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

#[cfg(feature = "gtk")]
impl Default for OutputsPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}

#[cfg(not(feature = "gtk"))]
pub struct OutputsPage;
