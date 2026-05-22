/// User interface module
pub mod pages;
#[cfg(feature = "gtk")]
pub mod window;
#[cfg(feature = "gtk")]
pub mod apply_bar;

#[cfg(feature = "gtk")]
pub use window::SwayConfigWindow;
#[cfg(feature = "gtk")]
pub use apply_bar::ApplyBar;
