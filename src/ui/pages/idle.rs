/// Idle configuration page (screensaver, lock)
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Idle page - for configuring idle behavior
pub struct IdlePage {
    widget: gtk4::Box,
}

impl IdlePage {
    /// Create a new idle page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Idle");
        status_page.set_description(Some("Configure screensaver and lock behavior"));
        
        widget.append(&status_page);
        
        IdlePage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for IdlePage {
    fn default() -> Self {
        Self::new()
    }
}
