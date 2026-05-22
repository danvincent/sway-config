/// Inputs configuration page (keyboard, mouse)
#[cfg(feature = "gtk")]
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "gtk")]
use crate::model::input::{KeyboardConfig, TouchpadConfig};

#[cfg(feature = "gtk")]
use crate::state::AppState;

/// Inputs page - for configuring input devices
#[cfg(feature = "gtk")]
pub struct InputsPage {
    widget: gtk4::Box,
    keyboards_list: gtk4::ListBox,
    touchpads_list: gtk4::ListBox,
    app_state: Rc<RefCell<AppState>>,
}

#[cfg(feature = "gtk")]
impl InputsPage {
    /// Create a new inputs page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        header_box.set_margin_start(12);
        header_box.set_margin_end(12);
        header_box.set_margin_top(12);
        header_box.set_margin_bottom(12);
        
        let title = gtk4::Label::new(Some("Input Devices"));
        title.add_css_class("title-1");
        header_box.append(&title);
        
        let description = gtk4::Label::new(Some("Configure keyboard, mouse, and touchpad settings"));
        description.add_css_class("dim-label");
        header_box.append(&description);
        
        widget.append(&header_box);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        
        // Keyboards section
        let keyboards_title = gtk4::Label::new(Some("Keyboards"));
        keyboards_title.add_css_class("heading");
        keyboards_title.set_halign(gtk4::Align::Start);
        main_box.append(&keyboards_title);
        
        let keyboards_list = gtk4::ListBox::new();
        keyboards_list.set_selection_mode(gtk4::SelectionMode::None);
        main_box.append(&keyboards_list);
        
        // Touchpads section
        let touchpads_title = gtk4::Label::new(Some("Touchpads"));
        touchpads_title.add_css_class("heading");
        touchpads_title.set_halign(gtk4::Align::Start);
        main_box.append(&touchpads_title);
        
        let touchpads_list = gtk4::ListBox::new();
        touchpads_list.set_selection_mode(gtk4::SelectionMode::None);
        main_box.append(&touchpads_list);
        
        scrolled.set_child(Some(&main_box));
        widget.append(&scrolled);
        
        InputsPage { widget, keyboards_list, touchpads_list, app_state }
    }

    /// Detect inputs and load them
    pub fn on_navigate(&self) {
        // Keyboards and touchpads are detected independently so that unsaved edits
        // in one subsection do not prevent detection from refreshing the other.
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
        
        // Load into UI from whatever is in AppState (user edits or detected)
        self.load_keyboards(&self.app_state.borrow().settings().keyboards);
        self.load_touchpads(&self.app_state.borrow().settings().touchpads);
    }

    /// Load keyboards into the list
    pub fn load_keyboards(&self, keyboards: &[KeyboardConfig]) {
        // Clear existing items
        while let Some(child) = self.keyboards_list.first_child() {
            self.keyboards_list.remove(&child);
        }
        
        // Add new items
        for keyboard in keyboards {
            let row = gtk4::ListBoxRow::new();
            let box_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            box_widget.set_margin_start(12);
            box_widget.set_margin_end(12);
            box_widget.set_margin_top(6);
            box_widget.set_margin_bottom(6);
            
            let name_label = gtk4::Label::new(Some(&keyboard.identifier));
            name_label.set_halign(gtk4::Align::Start);
            name_label.add_css_class("heading");
            box_widget.append(&name_label);
            
            let layout_label = gtk4::Label::new(Some(&format!("Layout: {}", keyboard.xkb_layout)));
            layout_label.set_halign(gtk4::Align::Start);
            layout_label.add_css_class("dim-label");
            box_widget.append(&layout_label);
            
            row.set_child(Some(&box_widget));
            self.keyboards_list.append(&row);
        }
    }

    /// Load touchpads into the list
    pub fn load_touchpads(&self, touchpads: &[TouchpadConfig]) {
        // Clear existing items
        while let Some(child) = self.touchpads_list.first_child() {
            self.touchpads_list.remove(&child);
        }
        
        // Add new items
        for touchpad in touchpads {
            let row = gtk4::ListBoxRow::new();
            let box_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            box_widget.set_margin_start(12);
            box_widget.set_margin_end(12);
            box_widget.set_margin_top(6);
            box_widget.set_margin_bottom(6);
            
            let name_label = gtk4::Label::new(Some(&touchpad.identifier));
            name_label.set_halign(gtk4::Align::Start);
            name_label.add_css_class("heading");
            box_widget.append(&name_label);
            
            let tap_label = gtk4::Label::new(Some(if touchpad.tap_to_click { "Tap enabled" } else { "Tap disabled" }));
            tap_label.set_halign(gtk4::Align::Start);
            tap_label.add_css_class("dim-label");
            box_widget.append(&tap_label);
            
            row.set_child(Some(&box_widget));
            self.touchpads_list.append(&row);
        }
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
