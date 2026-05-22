/// Autostart configuration page
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::model::autostart::AutostartConfig;
use crate::state::AppState;

/// Autostart page - for managing autostart programs
pub struct AutostartPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
    app_state: Rc<RefCell<AppState>>,
}

impl AutostartPage {
    /// Create a new autostart page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        
        let title_label = gtk4::Label::new(Some("Autostart Programs"));
        title_label.set_css_classes(&["title-2"]);
        title_label.set_halign(gtk4::Align::Start);
        vbox.append(&title_label);
        
        let subtitle = gtk4::Label::new(Some("Programs to launch automatically on startup"));
        subtitle.set_css_classes(&["subtitle"]);
        subtitle.set_halign(gtk4::Align::Start);
        vbox.append(&subtitle);
        
        // List box for entries
        let list_box = gtk4::ListBox::new();
        list_box.set_css_classes(&["boxed-list"]);
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        vbox.append(&list_box);
        
        // Add/Remove button box
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let _add_btn = gtk4::Button::with_label("Add");
        let _remove_btn = gtk4::Button::with_label("Remove");
        button_box.append(&_add_btn);
        button_box.append(&_remove_btn);
        vbox.append(&button_box);
        
        clamp.set_child(Some(&vbox));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);
        
        AutostartPage { widget, list_box, app_state }
    }
    
    /// Navigate to this page - load autostart config from app_state
    pub fn on_navigate(&self) {
        let config = self.app_state.borrow().settings().autostart.clone();
        self.load_autostart(&config);
    }
    
    /// Load autostart configuration into the page
    pub fn load_autostart(&self, config: &AutostartConfig) {
        // Clear existing entries
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Add entries from config
        for entry in &config.entries {
            let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            row.set_margin_top(6);
            row.set_margin_bottom(6);
            row.set_margin_start(12);
            row.set_margin_end(12);
            
            // Command and description label
            let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            let cmd_label = gtk4::Label::new(Some(&entry.command));
            cmd_label.set_halign(gtk4::Align::Start);
            cmd_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            let desc_label = gtk4::Label::new(Some(&entry.description));
            desc_label.set_css_classes(&["subtitle"]);
            desc_label.set_halign(gtk4::Align::Start);
            vbox.append(&cmd_label);
            vbox.append(&desc_label);
            
            // Enabled switch
            let switch = gtk4::Switch::new();
            switch.set_active(entry.enabled);
            switch.set_valign(gtk4::Align::Center);
            
            row.append(&vbox);
            row.set_hexpand(true);
            row.append(&switch);
            
            self.list_box.append(&row);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for AutostartPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
