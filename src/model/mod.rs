/// Model module - core data structures
pub mod settings;
pub mod theme;
pub mod output;
pub mod input;
pub mod idle;
pub mod waybar;
pub mod autostart;
pub mod notifications;
pub mod general;

pub use settings::Settings;
pub use theme::ThemeSelection;
pub use output::OutputConfig;
pub use input::{KeyboardConfig, TouchpadConfig};
pub use idle::IdleConfig;
pub use waybar::{WaybarConfig, BarPosition};
pub use autostart::AutostartConfig;
pub use notifications::NotificationsConfig;
