/// Themes configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Themes page - for configuring themes
pub struct ThemesPage {
    widget: gtk4::Box,
}

impl ThemesPage {
    /// Create a new themes page
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let status_page = libadwaita::StatusPage::new();
        status_page.set_title("Themes");
        status_page.set_description(Some("Manage and select themes"));
        
        widget.append(&status_page);
        
        ThemesPage { widget }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for ThemesPage {
    fn default() -> Self {
        Self::new()
    }
}
