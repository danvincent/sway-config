/// Main application window
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use crate::config::render::render_and_apply;
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
    /// ToastOverlay wrapping the content area for in-app notifications
    toast_overlay: libadwaita::ToastOverlay,
    /// Path to the settings store (used for Revert)
    store_path: PathBuf,
    /// Current page (tracks which page is displayed)
    current_page: Rc<RefCell<String>>,
    /// Stack widget for managing visible page
    stack: gtk4::Stack,
    /// List box for navigation
    list_box: gtk4::ListBox,
    /// Page structs stored to keep them alive and allow method calls
    outputs_page: Rc<OutputsPage>,
    inputs_page: Rc<InputsPage>,
    idle_page: Rc<IdlePage>,
    waybar_page: Rc<WaybarPage>,
    autostart_page: Rc<AutostartPage>,
    notifications_page: Rc<NotificationsPage>,
    themes_page: Rc<ThemesPage>,
    general_page: Rc<GeneralPage>,
}

impl SwayConfigWindow {
    /// Create a new window for the given application with settings from the store
    pub fn new(app: &libadwaita::Application, store: SettingsStore) -> Self {
        let window = libadwaita::ApplicationWindow::new(app);
        window.set_title(Some("Sway Configurator"));
        window.set_default_size(1200, 700);

        let store_path = store.path.clone();
        let app_state = Rc::new(RefCell::new(AppState::new(store.settings)));

        // Create the apply bar early so the dirty listener can reference it
        let apply_bar = ApplyBar::new();

        // Register dirty listener — shows the apply bar the moment any page marks dirty
        {
            let bar = apply_bar.clone();
            app_state.borrow_mut().set_dirty_listener(move || {
                bar.set_visible(true);
            });
        }

        // Create the main navigation split view
        let split_view = libadwaita::NavigationSplitView::new();
        split_view.set_sidebar_width_unit(libadwaita::LengthUnit::Sp);
        split_view.set_max_sidebar_width(200.0);

        // Create sidebar with page list
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        // Add page items in order: Outputs, Inputs, Idle, Waybar, Autostart, Notifications, Themes, General
        let page_configs = vec![
            ("Outputs", "outputs"),
            ("Inputs", "inputs"),
            ("Idle", "idle"),
            ("Waybar", "waybar"),
            ("Autostart", "autostart"),
            ("Notifications", "notifications"),
            ("Themes", "themes"),
            ("General", "general"),
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
        let outputs_page = Rc::new(OutputsPage::new(app_state.clone()));
        stack.add_named(outputs_page.widget(), Some("outputs"));

        let inputs_page = Rc::new(InputsPage::new(app_state.clone()));
        stack.add_named(inputs_page.widget(), Some("inputs"));

        let idle_page = Rc::new(IdlePage::new(app_state.clone()));
        stack.add_named(idle_page.widget(), Some("idle"));

        let waybar_page = Rc::new(WaybarPage::new(app_state.clone()));
        stack.add_named(waybar_page.widget(), Some("waybar"));

        let autostart_page = Rc::new(AutostartPage::new(app_state.clone()));
        stack.add_named(autostart_page.widget(), Some("autostart"));

        let notifications_page = Rc::new(NotificationsPage::new(app_state.clone()));
        stack.add_named(notifications_page.widget(), Some("notifications"));

        let themes_page = Rc::new(ThemesPage::new(app_state.clone()));
        stack.add_named(themes_page.widget(), Some("themes"));

        let general_page = Rc::new(GeneralPage::new(app_state.clone()));
        stack.add_named(general_page.widget(), Some("general"));

        // Set initial visible page
        stack.set_visible_child_name("outputs");

        content_box.append(&stack);

        // Add apply bar at the bottom (hidden initially)
        let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        content_box.append(&separator);
        content_box.append(apply_bar.widget());

        // Wrap content in a ToastOverlay for in-app notifications
        let toast_overlay = libadwaita::ToastOverlay::new();
        toast_overlay.set_child(Some(&content_box));

        // Create detail page
        let detail_page = libadwaita::NavigationPage::new(&toast_overlay, "Settings");
        split_view.set_content(Some(&detail_page));

        // Set the split view as window content
        window.set_content(Some(&split_view));

        let window_obj = SwayConfigWindow {
            window,
            app_state,
            apply_bar,
            toast_overlay,
            store_path,
            current_page: Rc::new(RefCell::new("outputs".to_string())),
            stack,
            list_box,
            outputs_page,
            inputs_page,
            idle_page,
            waybar_page,
            autostart_page,
            notifications_page,
            themes_page,
            general_page,
        };

        // Wire up navigation and apply/revert
        window_obj.setup_navigation();
        window_obj.setup_apply_revert();

        // Setup initial state visibility
        window_obj.update_apply_bar_visibility();
        
        // Select the first row by default and trigger on_navigate
        if let Some(first_row) = window_obj.list_box.row_at_index(0) {
            window_obj.list_box.select_row(Some(&first_row));
            window_obj.outputs_page.on_navigate();
            window_obj.update_apply_bar_visibility();
        }

        window_obj
    }

    /// Setup navigation signal handlers
    fn setup_navigation(&self) {
        // Wire list_box row-selected to switch stack
        {
            let stack_clone = self.stack.clone();
            let current_page_clone = self.current_page.clone();
            let page_ids = crate::ui::pages::page_ids();

            self.list_box.connect_row_selected(move |_list_box, row| {
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

        // Wire stack visible-child-name change → call on_navigate on the newly visible page
        {
            let outputs_page = Rc::clone(&self.outputs_page);
            let inputs_page = Rc::clone(&self.inputs_page);
            let idle_page = Rc::clone(&self.idle_page);
            let waybar_page = Rc::clone(&self.waybar_page);
            let autostart_page = Rc::clone(&self.autostart_page);
            let notifications_page = Rc::clone(&self.notifications_page);
            let themes_page = Rc::clone(&self.themes_page);
            let general_page = Rc::clone(&self.general_page);
            let apply_bar = self.apply_bar.clone();
            let app_state = self.app_state.clone();

            self.stack.connect_notify_local(Some("visible-child-name"), move |stack, _| {
                if let Some(name) = stack.visible_child_name() {
                    match name.as_str() {
                        "outputs" => outputs_page.on_navigate(),
                        "inputs" => inputs_page.on_navigate(),
                        "idle" => idle_page.on_navigate(),
                        "waybar" => waybar_page.on_navigate(),
                        "autostart" => autostart_page.on_navigate(),
                        "notifications" => notifications_page.on_navigate(),
                        "themes" => themes_page.on_navigate(),
                        "general" => general_page.on_navigate(),
                        _ => {}
                    }
                }
                // refresh apply bar after any state changes from on_navigate
                apply_bar.set_visible(app_state.borrow().is_dirty());
            });
        }
    }

    /// Wire Apply and Revert button signals
    fn setup_apply_revert(&self) {
        // Apply: render → write sway config files → save settings.toml → show toast → mark clean
        {
            let app_state = self.app_state.clone();
            let toast_overlay = self.toast_overlay.clone();
            let apply_bar = self.apply_bar.clone();
            let store_path = self.store_path.clone();

            self.apply_bar.button_apply().connect_clicked(move |_| {
                let result = {
                    let state = app_state.borrow();
                    render_and_apply(&state)
                };

                if result.success {
                    // Persist settings.toml so Revert can return to this state
                    let settings = app_state.borrow().settings().clone();
                    let store = SettingsStore { path: store_path.clone(), settings };
                    if let Err(e) = store.save() {
                        let msg = format!("Applied to sway but failed to save settings: {e}");
                        toast_overlay.add_toast(libadwaita::Toast::new(&msg));
                        return;
                    }

                    app_state.borrow_mut().mark_clean();
                    apply_bar.set_visible(false);

                    let msg = if result.errors.is_empty() {
                        "Configuration applied successfully".to_string()
                    } else {
                        // success=true but some reload/restart steps had non-fatal errors
                        format!("Applied with warnings: {}", result.errors.join("; "))
                    };
                    toast_overlay.add_toast(libadwaita::Toast::new(&msg));
                } else {
                    let msg = if result.errors.is_empty() {
                        "Failed to apply configuration".to_string()
                    } else {
                        format!("Error: {}", result.errors.join("; "))
                    };
                    toast_overlay.add_toast(libadwaita::Toast::new(&msg));
                }
            });
        }

        // Revert: reload settings.toml → replace in-memory settings → refresh current page → mark clean
        {
            let app_state = self.app_state.clone();
            let store_path = self.store_path.clone();
            let toast_overlay = self.toast_overlay.clone();
            let apply_bar = self.apply_bar.clone();
            let current_page = self.current_page.clone();
            let outputs_page = Rc::clone(&self.outputs_page);
            let inputs_page = Rc::clone(&self.inputs_page);
            let idle_page = Rc::clone(&self.idle_page);
            let waybar_page = Rc::clone(&self.waybar_page);
            let autostart_page = Rc::clone(&self.autostart_page);
            let notifications_page = Rc::clone(&self.notifications_page);
            let themes_page = Rc::clone(&self.themes_page);
            let general_page = Rc::clone(&self.general_page);

            self.apply_bar.button_revert().connect_clicked(move |_| {
                let fresh_settings = match SettingsStore::load(&store_path) {
                    Ok(store) => store.settings,
                    Err(e) => {
                        let msg = format!("Failed to reload settings: {e}");
                        toast_overlay.add_toast(libadwaita::Toast::new(&msg));
                        return;
                    }
                };

                app_state.borrow_mut().replace_settings(fresh_settings);

                // Before calling on_navigate() on Outputs/Inputs, set per-section dirty flags
                // so should_refresh_*() returns false and hardware re-detection is skipped.
                // This ensures on_navigate() reloads from the just-reverted settings.toml values
                // rather than overwriting them with freshly detected hardware state.
                app_state.borrow_mut().mark_outputs_dirty();
                app_state.borrow_mut().mark_keyboards_dirty();
                app_state.borrow_mut().mark_touchpads_dirty();

                // Refresh the currently visible page
                match current_page.borrow().as_str() {
                    "outputs" => outputs_page.on_navigate(),
                    "inputs" => inputs_page.on_navigate(),
                    "idle" => idle_page.on_navigate(),
                    "waybar" => waybar_page.on_navigate(),
                    "autostart" => autostart_page.on_navigate(),
                    "notifications" => notifications_page.on_navigate(),
                    "themes" => themes_page.on_navigate(),
                    "general" => general_page.on_navigate(),
                    _ => {}
                }

                // Now safe to mark clean — dirty flags are cleared, apply bar hidden
                app_state.borrow_mut().mark_clean();
                apply_bar.set_visible(false);
            });
        }
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
