/// Main application window
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::store::SettingsStore;
use crate::state::AppState;
use crate::ui::pages::*;
use crate::ui::apply_bar::ApplyBar;

/// Main application window with sidebar navigation and content area
pub struct SwayConfigWindow {
    /// The underlying GTK window
    window: libadwaita::ApplicationWindow,
    /// Shared application state
    app_state: Rc<RefCell<AppState>>,
    /// Apply bar for unsaved changes
    apply_bar: ApplyBar,
    /// Current page (tracks which page is displayed)
    current_page: Rc<RefCell<String>>,
    /// Stack widget for managing visible page
    stack: gtk4::Stack,
    /// List box for navigation
    list_box: gtk4::ListBox,
}

impl SwayConfigWindow {
    /// Create a new window for the given application with settings from the store
    pub fn new(app: &libadwaita::Application, store: SettingsStore) -> Self {
        let window = libadwaita::ApplicationWindow::new(app);
        window.set_title(Some("Sway Configurator"));
        window.set_default_size(1200, 700);

        let app_state = Rc::new(RefCell::new(AppState::new(store.settings)));

        // Create the main navigation split view
        let split_view = libadwaita::NavigationSplitView::new();
        split_view.set_sidebar_width_unit(libadwaita::LengthUnit::Sp);
        split_view.set_max_sidebar_width(200.0);

        // Create sidebar with page list
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        // Add page items in order: Outputs, Inputs, Idle, Waybar, Autostart, Notifications, Themes
        let page_configs = vec![
            ("Outputs", "outputs"),
            ("Inputs", "inputs"),
            ("Idle", "idle"),
            ("Waybar", "waybar"),
            ("Autostart", "autostart"),
            ("Notifications", "notifications"),
            ("Themes", "themes"),
        ];

        for (label, _id) in &page_configs {
            let row = gtk4::ListBoxRow::new();
            let label_widget = gtk4::Label::new(Some(label));
            label_widget.set_margin_top(12);
            label_widget.set_margin_bottom(12);
            label_widget.set_margin_start(12);
            label_widget.set_margin_end(12);
            row.set_child(Some(&label_widget));
            list_box.append(&row);
        }

        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sidebar_box.append(&list_box);
        let sidebar_page = libadwaita::NavigationPage::new(&sidebar_box, "Pages");
        split_view.set_sidebar(Some(&sidebar_page));

        // Create content area with stack for page switching
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_box.set_hexpand(true);
        content_box.set_vexpand(true);

        // Create the stack to hold all pages
        let stack = gtk4::Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(200);

        // Create all page widgets and add to stack
        let outputs_page = OutputsPage::new();
        stack.add_named(outputs_page.widget(), Some("outputs"));

        let inputs_page = InputsPage::new();
        stack.add_named(inputs_page.widget(), Some("inputs"));

        let idle_page = IdlePage::new();
        stack.add_named(idle_page.widget(), Some("idle"));

        let waybar_page = WaybarPage::new();
        stack.add_named(waybar_page.widget(), Some("waybar"));

        let autostart_page = AutostartPage::new();
        stack.add_named(autostart_page.widget(), Some("autostart"));

        let notifications_page = NotificationsPage::new();
        stack.add_named(notifications_page.widget(), Some("notifications"));

        let themes_page = ThemesPage::new();
        stack.add_named(themes_page.widget(), Some("themes"));

        // Set initial visible page
        stack.set_visible_child_name("outputs");

        content_box.append(&stack);

        // Create the apply bar
        let apply_bar = ApplyBar::new();

        // Add apply bar at the bottom
        let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        content_box.append(&separator);
        content_box.append(apply_bar.widget());

        // Create detail page
        let detail_page = libadwaita::NavigationPage::new(&content_box, "Settings");
        split_view.set_content(Some(&detail_page));

        // Set the split view as window content
        window.set_content(Some(&split_view));

        let window_obj = SwayConfigWindow {
            window,
            app_state,
            apply_bar,
            current_page: Rc::new(RefCell::new("outputs".to_string())),
            stack,
            list_box,
        };

        // Wire up the list box row-selected signal to switch pages
        {
            let stack_clone = window_obj.stack.clone();
            let current_page_clone = window_obj.current_page.clone();
            let page_ids = crate::ui::pages::page_ids();

            window_obj.list_box.connect_row_selected(move |_list_box, row| {
                if let Some(row) = row {
                    let index = row.index();
                    if index >= 0 {
                        let index = index as usize;
                        if index < page_ids.len() {
                            let page_id = page_ids[index];
                            stack_clone.set_visible_child_name(page_id);
                            *current_page_clone.borrow_mut() = page_id.to_string();
                        }
                    }
                }
            });
        }

        // Setup initial state visibility
        window_obj.update_apply_bar_visibility();
        
        // Select the first row by default
        if let Some(first_row) = window_obj.list_box.row_at_index(0) {
            window_obj.list_box.select_row(Some(&first_row));
        }

        window_obj
    }

    /// Navigate to a specific page
    pub fn navigate_to(&self, page_id: &str) {
        *self.current_page.borrow_mut() = page_id.to_string();
        self.stack.set_visible_child_name(page_id);
        
        // Select the corresponding row in the list box
        let page_ids = crate::ui::pages::page_ids();
        if let Some(index) = page_ids.iter().position(|&id| id == page_id) {
            if let Some(row) = self.list_box.row_at_index(index as i32) {
                self.list_box.select_row(Some(&row));
            }
        }
    }

    /// Get a reference to the window
    pub fn window(&self) -> &libadwaita::ApplicationWindow {
        &self.window
    }

    /// Get the shared app state
    pub fn app_state(&self) -> Rc<RefCell<AppState>> {
        self.app_state.clone()
    }

    /// Present the window
    pub fn present(&self) {
        self.window.present();
    }

    /// Update apply bar visibility based on dirty state
    fn update_apply_bar_visibility(&self) {
        let state = self.app_state.borrow();
        self.apply_bar.set_visible_if_dirty(state.is_dirty());
    }

    /// Mark state as dirty and update UI
    pub fn mark_dirty(&self) {
        self.app_state.borrow_mut().mark_dirty();
        self.update_apply_bar_visibility();
    }

    /// Mark state as clean and update UI
    pub fn mark_clean(&self) {
        self.app_state.borrow_mut().mark_clean();
        self.update_apply_bar_visibility();
    }
}
