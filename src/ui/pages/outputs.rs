/// Outputs configuration page (displays/monitors)
#[cfg(feature = "gtk")]
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "gtk")]
use crate::model::output::OutputConfig;

#[cfg(feature = "gtk")]
use crate::state::AppState;

/// Outputs page - for configuring monitor layout
#[cfg(feature = "gtk")]
pub struct OutputsPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
    app_state: Rc<RefCell<AppState>>,
}

#[cfg(feature = "gtk")]
impl OutputsPage {
    /// Create a new outputs page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        header_box.set_margin_start(12);
        header_box.set_margin_end(12);
        header_box.set_margin_top(12);
        header_box.set_margin_bottom(12);
        
        let title = gtk4::Label::new(Some("Displays"));
        title.add_css_class("title-1");
        header_box.append(&title);
        
        let description = gtk4::Label::new(Some("Configure monitor layout and settings"));
        description.add_css_class("dim-label");
        header_box.append(&description);
        
        widget.append(&header_box);
        
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_child(Some(&list_box));
        scrolled.set_vexpand(true);
        
        widget.append(&scrolled);
        
        OutputsPage { widget, list_box, app_state }
    }

    /// Detect outputs and load them
    pub fn on_navigate(&self) {
        // Only refresh from hardware if outputs have no unsaved user edits.
        // Edits in other sections (e.g. keyboards) must not suppress detection here.
        if self.app_state.borrow().should_refresh_outputs() {
            // Detect outputs from sway
            let detected = crate::config::detect::detect_outputs();
            
            // Convert to OutputConfig
            let configs: Vec<OutputConfig> = detected.iter()
                .map(|out| OutputConfig::from_sway(out))
                .collect();
            
            // Update app state (refresh without marking dirty)
            self.app_state.borrow_mut().refresh_outputs(configs);
        }
        
        // Load into UI from whatever is in AppState (user edits or detected)
        self.load_outputs(&self.app_state.borrow().settings().outputs);
    }

    /// Load outputs into the list
    pub fn load_outputs(&self, outputs: &[OutputConfig]) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Add new items
        for output in outputs {
            let row = gtk4::ListBoxRow::new();
            let box_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            box_widget.set_margin_start(12);
            box_widget.set_margin_end(12);
            box_widget.set_margin_top(6);
            box_widget.set_margin_bottom(6);
            
            let name_label = gtk4::Label::new(Some(&output.name));
            name_label.set_halign(gtk4::Align::Start);
            name_label.add_css_class("heading");
            box_widget.append(&name_label);
            
            let status = if output.enabled { "Enabled" } else { "Disabled" };
            let status_label = gtk4::Label::new(Some(status));
            status_label.set_halign(gtk4::Align::Start);
            status_label.add_css_class("dim-label");
            box_widget.append(&status_label);
            
            row.set_child(Some(&box_widget));
            self.list_box.append(&row);
        }
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
