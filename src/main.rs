#[cfg(feature = "gtk")]
use libadwaita::prelude::*;
#[cfg(feature = "gtk")]
use sway_configurator::config::store::SettingsStore;
#[cfg(feature = "gtk")]
use sway_configurator::ui::SwayConfigWindow;

#[cfg(feature = "gtk")]
fn main() {
    tracing_subscriber::fmt::init();

    let app = libadwaita::Application::new(Some("com.github.danvincent.sway-configurator"), Default::default());
    
    app.connect_activate(|app| {
        let store = if std::env::var("SWAY_CONFIGURATOR_TEST").as_deref() == Ok("1") {
            let test_dir = std::env::temp_dir().join("sway-configurator-test-run");
            std::fs::create_dir_all(&test_dir)
                .expect("Failed to create test directory");
            tracing::info!("Test mode: using {}", test_dir.display());
            SettingsStore::test_store(&test_dir)
        } else {
            match SettingsStore::open_default() {
                Ok(store) => store,
                Err(err) => {
                    eprintln!("Failed to load settings: {}", err);
                    return;
                }
            }
        };

        let window = SwayConfigWindow::new(&app, store);
        window.present();
    });

    let ret = app.run();
    std::process::exit(ret.into());
}

#[cfg(not(feature = "gtk"))]
fn main() {
    eprintln!("Sway Configurator - GTK feature not enabled");
    eprintln!("Please build with: cargo build --features gtk");
}
