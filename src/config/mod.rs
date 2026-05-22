/// Config module - persistence and rendering
pub mod store;
pub mod read_helpers;
pub mod render_model;
pub mod swaymsg;
pub mod detect;
pub mod feature;
pub mod apply;
pub mod render;

pub use store::SettingsStore;
pub use read_helpers::*;
pub use render_model::RenderModel;
pub use swaymsg::SwayError;
pub use detect::{SwayOutput, SwayInput};
pub use feature::*;
pub use apply::{apply, ApplyConfig, ApplyResult};
pub use render::{render_and_apply, render_dry_run};
