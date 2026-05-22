/// Apply bar widget for unsaved changes
use gtk4::prelude::*;
use gtk4::{Align, Orientation};

/// ApplyBar - shows unsaved changes notification and action buttons
pub struct ApplyBar {
    widget: gtk4::Box,
    button_apply: gtk4::Button,
    button_revert: gtk4::Button,
}

impl ApplyBar {
    /// Create a new apply bar
    pub fn new() -> Self {
        let widget = gtk4::Box::new(Orientation::Horizontal, 12);
        widget.set_margin_top(12);
        widget.set_margin_bottom(12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_halign(Align::End);

        // Unsaved changes label
        let label = gtk4::Label::new(Some("Unsaved changes"));
        label.set_halign(Align::Start);
        label.set_hexpand(true);

        // Apply button
        let button_apply = gtk4::Button::with_label("Apply");
        button_apply.add_css_class("suggested-action");

        // Revert button
        let button_revert = gtk4::Button::with_label("Revert");

        widget.append(&label);
        widget.append(&button_revert);
        widget.append(&button_apply);

        ApplyBar {
            widget,
            button_apply,
            button_revert,
        }
    }

    /// Get a reference to the apply bar's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }

    /// Set visibility based on dirty state
    pub fn set_visible_if_dirty(&self, dirty: bool) {
        self.widget.set_visible(dirty);
    }

    /// Get a reference to the apply button
    pub fn button_apply(&self) -> &gtk4::Button {
        &self.button_apply
    }

    /// Get a reference to the revert button
    pub fn button_revert(&self) -> &gtk4::Button {
        &self.button_revert
    }
}

impl Default for ApplyBar {
    fn default() -> Self {
        Self::new()
    }
}
