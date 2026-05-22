/// Notifications configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Notifications page - for configuring notifications
pub struct NotificationsPage {
    widget: gtk4::Box,
}

impl NotificationsPage {
    /// Create a new notifications page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Notifications");
        status_page.set_description(Some("Configure notification daemon settings"));
        
        widget.append(&status_page);
        
        NotificationsPage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for NotificationsPage {
    fn default() -> Self {
        Self::new()
    }
}
