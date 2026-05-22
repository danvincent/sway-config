/// Outputs configuration page (displays/monitors)
#[cfg(feature = "gtk")]
use gtk4::prelude::*;
#[cfg(feature = "gtk")]
use libadwaita::prelude::*;

#[cfg(feature = "gtk")]
use crate::model::output::OutputConfig;

/// Outputs page - for configuring monitor layout
#[cfg(feature = "gtk")]
pub struct OutputsPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
}

#[cfg(feature = "gtk")]
impl OutputsPage {
    /// Create a new outputs page
    pub fn new() -> Self {
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
        
        OutputsPage { widget, list_box }
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
        Self::new()
    }
}

#[cfg(not(feature = "gtk"))]
pub struct OutputsPage;
