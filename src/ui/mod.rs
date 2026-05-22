#[cfg(feature = "gtk")]
pub mod apply_bar;
/// User interface module
pub mod pages;
#[cfg(feature = "gtk")]
pub mod window;

#[cfg(feature = "gtk")]
pub use apply_bar::ApplyBar;
#[cfg(feature = "gtk")]
pub use window::SwayConfigWindow;
