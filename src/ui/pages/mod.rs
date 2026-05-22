pub mod autostart;
#[cfg(feature = "gtk")]
pub mod general;
#[cfg(feature = "gtk")]
pub mod idle;
pub mod inputs;
pub mod notifications;
/// Pages module - UI pages for different configuration sections
pub mod outputs;
#[cfg(feature = "gtk")]
pub mod themes;
pub mod waybar;

#[cfg(feature = "gtk")]
pub use autostart::AutostartPage;
#[cfg(feature = "gtk")]
pub use general::GeneralPage;
#[cfg(feature = "gtk")]
pub use idle::IdlePage;
#[cfg(feature = "gtk")]
pub use inputs::InputsPage;
#[cfg(feature = "gtk")]
pub use notifications::NotificationsPage;
#[cfg(feature = "gtk")]
pub use outputs::OutputsPage;
#[cfg(feature = "gtk")]
pub use themes::ThemesPage;
#[cfg(feature = "gtk")]
pub use waybar::WaybarPage;

/// Returns page IDs in sidebar display order.
/// This function is not gated by cfg so tests can call it without GTK.
pub fn page_ids() -> &'static [&'static str] {
    &[
        "outputs",
        "inputs",
        "idle",
        "waybar",
        "autostart",
        "notifications",
        "themes",
        "general",
    ]
}
