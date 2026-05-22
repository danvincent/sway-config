/// Config module - persistence and rendering
pub mod store;
pub mod render_model;
pub mod swaymsg;
pub mod detect;
pub mod feature;

pub use store::SettingsStore;
pub use render_model::RenderModel;
pub use swaymsg::SwayError;
pub use detect::{SwayOutput, SwayInput};
pub use feature::*;
