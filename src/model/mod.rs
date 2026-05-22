pub mod autostart;
pub mod general;
pub mod idle;
pub mod input;
pub mod notifications;
pub mod output;
/// Model module - core data structures
pub mod settings;
pub mod theme;
pub mod waybar;

pub use autostart::AutostartConfig;
pub use idle::IdleConfig;
pub use input::{KeyboardConfig, TouchpadConfig};
pub use notifications::NotificationsConfig;
pub use output::OutputConfig;
pub use settings::Settings;
pub use theme::ThemeSelection;
pub use waybar::{BarPosition, WaybarConfig};
