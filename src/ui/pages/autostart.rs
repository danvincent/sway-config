/// Autostart configuration page
use crate::model::autostart::AutostartEntry;

// ── Pure helpers ──────────────────────────────────────────────────────────────

/// Create a new autostart entry with a generated id.
pub fn make_entry(command: &str, description: &str) -> AutostartEntry {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    AutostartEntry {
        id: format!("{}-{}", command.replace(' ', "_"), ts),
        command: command.to_string(),
        description: description.to_string(),
        enabled: true,
    }
}

/// Toggle an entry's enabled state by id. Returns true if found.
pub fn toggle_entry(entries: &mut Vec<AutostartEntry>, id: &str, enabled: bool) -> bool {
    if let Some(e) = entries.iter_mut().find(|e| e.id == id) {
        e.enabled = enabled;
        true
    } else {
        false
    }
}

/// Remove an entry by id. Returns true if removed.
pub fn remove_entry(entries: &mut Vec<AutostartEntry>, id: &str) -> bool {
    let before = entries.len();
    entries.retain(|e| e.id != id);
    entries.len() < before
}

// ── GTK page ──────────────────────────────────────────────────────────────────

#[cfg(feature = "gtk")]
use crate::model::autostart::AutostartConfig;
#[cfg(feature = "gtk")]
use crate::state::AppState;
#[cfg(feature = "gtk")]
use gtk4::prelude::*;
#[cfg(feature = "gtk")]
use libadwaita::prelude::*;
#[cfg(feature = "gtk")]
use std::cell::RefCell;
#[cfg(feature = "gtk")]
use std::rc::Rc;

/// Autostart page - for managing autostart programs
#[cfg(feature = "gtk")]
pub struct AutostartPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
    #[allow(dead_code)]
    add_button: gtk4::Button,
    app_state: Rc<RefCell<AppState>>,
}

#[cfg(feature = "gtk")]
impl AutostartPage {
    /// Create a new autostart page
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);

        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);

        let outer_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        outer_box.set_margin_start(12);
        outer_box.set_margin_end(12);
        outer_box.set_margin_top(12);
        outer_box.set_margin_bottom(12);

        // ── Main group ─────────────────────────────────────────────────────────
        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Autostart Programs");
        group.set_description(Some("Programs launched automatically when Sway starts"));

        let add_button = gtk4::Button::with_label("Add Program");
        add_button.set_css_classes(&["suggested-action"]);
        add_button.set_halign(gtk4::Align::End);
        group.set_header_suffix(Some(&add_button));

        let list_box = gtk4::ListBox::new();
        list_box.set_css_classes(&["boxed-list"]);
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        group.add(&list_box);

        outer_box.append(&group);
        clamp.set_child(Some(&outer_box));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        // ── Add button signal ──────────────────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let list_box_ref = list_box.clone();
            let widget_ref = widget.clone();
            add_button.connect_clicked(move |_| {
                Self::show_add_dialog(&widget_ref, Rc::clone(&state), list_box_ref.clone());
            });
        }

        AutostartPage {
            widget,
            list_box,
            add_button,
            app_state,
        }
    }

    /// Show an add-program dialog and, on confirm, add the entry to state + UI.
    fn show_add_dialog(parent: &gtk4::Box, state: Rc<RefCell<AppState>>, list_box: gtk4::ListBox) {
        let dialog = libadwaita::AlertDialog::new(
            Some("Add Autostart Program"),
            Some("Enter the command to run at startup"),
        );
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("add", "Add");
        dialog.set_response_appearance("add", libadwaita::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("add"));
        dialog.set_close_response("cancel");

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.set_margin_top(8);

        let cmd_entry = libadwaita::EntryRow::new();
        cmd_entry.set_title("Command");
        let desc_entry = libadwaita::EntryRow::new();
        desc_entry.set_title("Description (optional)");

        let group = libadwaita::PreferencesGroup::new();
        group.add(&cmd_entry);
        group.add(&desc_entry);
        content.append(&group);

        dialog.set_extra_child(Some(&content));

        {
            let cmd_entry = cmd_entry.clone();
            let desc_entry = desc_entry.clone();
            let parent_widget = parent.upcast_ref::<gtk4::Widget>().clone();
            dialog.connect_response(None, move |_, response| {
                if response != "add" {
                    return;
                }
                let cmd = cmd_entry.text().to_string();
                let cmd = cmd.trim().to_string();
                if cmd.is_empty() {
                    return;
                }
                let desc = desc_entry.text().to_string();
                let entry = make_entry(&cmd, &desc);
                // Append row to UI
                let row = Self::build_row(&entry, Rc::clone(&state), list_box.clone());
                list_box.append(&row);
                // Update state
                state
                    .borrow_mut()
                    .settings_mut()
                    .autostart
                    .entries
                    .push(entry);
                state.borrow_mut().mark_dirty();
                let _ = parent_widget; // keep alive
            });
        }

        // Present the dialog attached to the nearest window ancestor
        let root = parent.root().and_downcast::<gtk4::Window>();
        dialog.present(root.as_ref());
    }

    /// Build one ActionRow for an autostart entry (with enable toggle + delete button).
    fn build_row(
        entry: &AutostartEntry,
        state: Rc<RefCell<AppState>>,
        list_box: gtk4::ListBox,
    ) -> libadwaita::ActionRow {
        let row = libadwaita::ActionRow::new();
        row.set_title(&entry.command);
        if !entry.description.is_empty() {
            row.set_subtitle(&entry.description);
        }

        // Enable toggle
        let toggle = gtk4::Switch::new();
        toggle.set_active(entry.enabled);
        toggle.set_valign(gtk4::Align::Center);
        row.add_suffix(&toggle);
        row.set_activatable_widget(Some(&toggle));

        // Delete button
        let delete_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_css_classes(&["flat", "destructive-action"]);
        delete_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&delete_btn);

        let entry_id = entry.id.clone();
        {
            let state = Rc::clone(&state);
            let id = entry_id.clone();
            toggle.connect_state_set(move |_, active| {
                let mut borrowed = state.borrow_mut();
                toggle_entry(&mut borrowed.settings_mut().autostart.entries, &id, active);
                borrowed.mark_dirty();
                gtk4::glib::Propagation::Proceed
            });
        }
        {
            let state = Rc::clone(&state);
            let id = entry_id.clone();
            let list_box = list_box.clone();
            let row_ref = row.clone();
            delete_btn.connect_clicked(move |_| {
                let mut borrowed = state.borrow_mut();
                remove_entry(&mut borrowed.settings_mut().autostart.entries, &id);
                borrowed.mark_dirty();
                drop(borrowed);
                list_box.remove(&row_ref);
            });
        }

        row
    }

    /// Navigate to this page - load autostart config from app_state
    pub fn on_navigate(&self) {
        let config = self.app_state.borrow().settings().autostart.clone();
        self.load_autostart(&config);
    }

    /// Load autostart configuration into the page
    pub fn load_autostart(&self, config: &AutostartConfig) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        for entry in &config.entries {
            let row = Self::build_row(entry, Rc::clone(&self.app_state), self.list_box.clone());
            self.list_box.append(&row);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

#[cfg(feature = "gtk")]
impl Default for AutostartPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
