/// Autostart configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Autostart page - for managing autostart programs
pub struct AutostartPage {
    widget: gtk4::Box,
}

impl AutostartPage {
    /// Create a new autostart page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Autostart");
        status_page.set_description(Some("Manage programs to launch on startup"));
        
        widget.append(&status_page);
        
        AutostartPage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for AutostartPage {
    fn default() -> Self {
        Self::new()
    }
}
