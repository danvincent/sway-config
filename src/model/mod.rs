/// Model module - core data structures
pub mod settings;
pub mod theme;
pub mod output;
pub mod input;

pub use settings::Settings;
pub use theme::ThemeSelection;
pub use output::OutputConfig;
pub use input::{KeyboardConfig, TouchpadConfig};
