#![cfg(feature = "gtk")]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;

use gtk4::prelude::*;
use sway_config::model::settings::Settings;
use sway_config::state::AppState;
use sway_config::ui::pages::{
    AutostartPage, GeneralPage, IdlePage, InputsPage, NotificationsPage, OutputsPage, ThemesPage,
    WaybarPage,
};
use sway_config::ui::SwayConfigWindow;

fn init_gtk() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        gtk4::init().expect("GTK init failed");
    });
}

#[test]
fn test_main_window_constructs_with_store() {
    init_gtk();

    let app = libadwaita::Application::new(
        Some("com.github.danvincent.sway-config.test"),
        Default::default(),
    );
    app.register(None::<&gtk4::gio::Cancellable>)
        .expect("app register failed");

    let tmp = tempfile::tempdir().expect("temp dir");
    let store = sway_config::config::store::SettingsStore::test_store(tmp.path());
    let window = SwayConfigWindow::new(&app, store);

    assert!(window.window().title().is_some());
}

#[test]
fn test_all_pages_construct_widgets() {
    init_gtk();

    let app_state = Rc::new(RefCell::new(AppState::new(Settings::default())));

    let outputs = OutputsPage::new(app_state.clone());
    let inputs = InputsPage::new(app_state.clone());
    let idle = IdlePage::new(app_state.clone());
    let waybar = WaybarPage::new(app_state.clone());
    let autostart = AutostartPage::new(app_state.clone());
    let notifications = NotificationsPage::new(app_state.clone());
    let themes = ThemesPage::new(app_state.clone());
    let general = GeneralPage::new(app_state);

    let _ = outputs.widget();
    let _ = inputs.widget();
    let _ = idle.widget();
    let _ = waybar.widget();
    let _ = autostart.widget();
    let _ = notifications.widget();
    let _ = themes.widget();
    let _ = general.widget();
}
