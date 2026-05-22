/// Main library entry point
pub mod app;
pub mod model;
pub mod config;
pub mod state;
pub mod ui;

pub use app::App;
pub use model::settings::Settings;
pub use config::store::SettingsStore;
pub use state::AppState;
