/// Inputs configuration page (keyboard, mouse)
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Inputs page - for configuring input devices
pub struct InputsPage {
    widget: gtk4::Box,
}

impl InputsPage {
    /// Create a new inputs page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Input Devices");
        status_page.set_description(Some("Configure keyboard, mouse, and touchpad settings"));
        
        widget.append(&status_page);
        
        InputsPage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for InputsPage {
    fn default() -> Self {
        Self::new()
    }
}
