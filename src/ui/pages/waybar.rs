/// Waybar configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Waybar page - for configuring the status bar
pub struct WaybarPage {
    widget: gtk4::Box,
}

impl WaybarPage {
    /// Create a new waybar page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Waybar");
        status_page.set_description(Some("Configure the waybar status bar"));
        
        widget.append(&status_page);
        
        WaybarPage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for WaybarPage {
    fn default() -> Self {
        Self::new()
    }
}
