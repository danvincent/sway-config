/// Outputs configuration page (displays/monitors)
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Outputs page - for configuring monitor layout
pub struct OutputsPage {
    widget: gtk4::Box,
}

impl OutputsPage {
    /// Create a new outputs page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Displays");
        status_page.set_description(Some("Configure monitor layout and settings"));
        
        widget.append(&status_page);
        
        OutputsPage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for OutputsPage {
    fn default() -> Self {
        Self::new()
    }
}
