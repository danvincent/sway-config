use crate::state::AppState;
/// General configuration page (terminal and other sway settings)
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// General page - for general Sway environment settings
pub struct GeneralPage {
    widget: gtk4::Box,
    terminal_row: libadwaita::ActionRow,
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

        let outer_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        outer_box.set_margin_start(12);
        outer_box.set_margin_end(12);
        outer_box.set_margin_top(12);
        outer_box.set_margin_bottom(12);

        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Sway Environment");
        group.set_description(Some("General Sway compositor settings"));

        let terminal_row = libadwaita::ActionRow::new();
        terminal_row.set_title("Default terminal");
        terminal_row.set_subtitle("Auto-detect (unset)");
        terminal_row.set_tooltip_text(Some(
            "Sets $term in sway config (e.g. alacritty, foot, kitty)",
        ));

        let edit_button = gtk4::Button::with_label("Edit");
        edit_button.set_valign(gtk4::Align::Center);
        terminal_row.add_suffix(&edit_button);
        terminal_row.set_activatable_widget(Some(&edit_button));
        group.add(&terminal_row);

        outer_box.append(&group);
        clamp.set_child(Some(&outer_box));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        {
            let state = Rc::clone(&app_state);
            let row_ref = terminal_row.clone();
            let parent = widget.clone();
            edit_button.connect_clicked(move |_| {
                Self::show_terminal_dialog(&parent, Rc::clone(&state), &row_ref);
            });
        }

        GeneralPage {
            widget,
            terminal_row,
            app_state,
        }
    }

    /// Navigate to this page - load general config from app_state
    pub fn on_navigate(&self) {
        let state = self.app_state.borrow();
        let value = state.settings().general.terminal.trim();
        if value.is_empty() {
            self.terminal_row.set_subtitle("Auto-detect (unset)");
        } else {
            self.terminal_row.set_subtitle(value);
        }
    }

    /// Get a reference to the page's widget
    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl GeneralPage {
    fn show_terminal_dialog(
        parent: &gtk4::Box,
        app_state: Rc<RefCell<AppState>>,
        terminal_row: &libadwaita::ActionRow,
    ) {
        let dialog = libadwaita::AlertDialog::new(
            Some("Set Default Terminal"),
            Some("Configure command and optional arguments"),
        );
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("clear", "Clear");
        dialog.add_response("save", "Save");
        dialog.set_response_appearance("save", libadwaita::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("save"));
        dialog.set_close_response("cancel");

        let current = app_state.borrow().settings().general.terminal.clone();
        let (current_cmd, current_args) = split_command_and_args(&current);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.set_margin_top(8);

        let cmd_entry = libadwaita::EntryRow::new();
        cmd_entry.set_title("Command");
        cmd_entry.set_text(&current_cmd);

        let args_entry = libadwaita::EntryRow::new();
        args_entry.set_title("Arguments (optional)");
        args_entry.set_text(&current_args);

        let group = libadwaita::PreferencesGroup::new();
        group.add(&cmd_entry);
        group.add(&args_entry);
        content.append(&group);
        dialog.set_extra_child(Some(&content));

        {
            let state = Rc::clone(&app_state);
            let row_ref = terminal_row.clone();
            dialog.connect_response(None, move |_, response| match response {
                "save" => {
                    let cmd = cmd_entry.text().trim().to_string();
                    let args = args_entry.text().trim().to_string();
                    if cmd.is_empty() {
                        return;
                    }
                    let terminal = if args.is_empty() {
                        cmd
                    } else {
                        format!("{} {}", cmd, args)
                    };
                    state.borrow_mut().settings_mut().general.terminal = terminal.clone();
                    state.borrow_mut().mark_dirty();
                    row_ref.set_subtitle(&terminal);
                }
                "clear" => {
                    state.borrow_mut().settings_mut().general.terminal.clear();
                    state.borrow_mut().mark_dirty();
                    row_ref.set_subtitle("Auto-detect (unset)");
                }
                _ => {}
            });
        }

        let root = parent.root().and_downcast::<gtk4::Window>();
        dialog.present(root.as_ref());
    }
}

fn split_command_and_args(value: &str) -> (String, String) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (String::new(), String::new());
    }
    if let Some(idx) = trimmed.find(char::is_whitespace) {
        let (cmd, args) = trimmed.split_at(idx);
        (cmd.trim().to_string(), args.trim().to_string())
    } else {
        (trimmed.to_string(), String::new())
    }
}

impl Default for GeneralPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}
