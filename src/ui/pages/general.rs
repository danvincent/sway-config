/// General configuration page (terminal and other sway settings)
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::state::AppState;

/// General page - for general Sway environment settings
pub struct GeneralPage {
    widget: gtk4::Box,
    terminal_entry: libadwaita::EntryRow,
    app_state: Rc<RefCell<AppState>>,
}

impl GeneralPage {
    /// Create a new general page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);
        
        let prefs_page = libadwaita::PreferencesPage::new();
        
        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Sway Environment");
        group.set_description(Some("General Sway compositor settings"));
        
        let terminal_entry = libadwaita::EntryRow::new();
        terminal_entry.set_title("Default terminal");
        terminal_entry.set_tooltip_text(Some("Sets $term in sway config (e.g. alacritty, foot, kitty)"));
        group.add(&terminal_entry);
        
        prefs_page.add(&group);
        clamp.set_child(Some(&prefs_page));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);
        
        GeneralPage { widget, terminal_entry, app_state }
    }
    
    /// Navigate to this page - load general config from app_state
    pub fn on_navigate(&self) {
        let state = self.app_state.borrow();
        self.terminal_entry.set_text(&state.settings().general.terminal);
    }
    
    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for GeneralPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
