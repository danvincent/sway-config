/// Waybar configuration - status bar settings
use serde::{Deserialize, Serialize};

/// Position of the waybar on screen
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarPosition {
    /// Top of screen
    #[default]
    Top,
    /// Bottom of screen
    Bottom,
    /// Left side of screen
    Left,
    /// Right side of screen
    Right,
}

/// A single waybar module with enabled/disabled state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaybarModule {
    /// Module name (e.g., "battery", "network", "pulseaudio")
    pub name: String,
    /// Whether this module is enabled
    pub enabled: bool,
}

impl WaybarModule {
    /// Create a new waybar module
    pub fn new(name: &str, enabled: bool) -> Self {
        WaybarModule {
            name: name.to_string(),
            enabled,
        }
    }
}

/// Waybar configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WaybarConfig {
    /// Enable/disable the waybar
    pub enabled: bool,
    /// Position of the bar on screen
    pub position: BarPosition,
    /// Height of the bar in pixels
    pub height: u32,
    /// Modules on the left side
    pub modules_left: Vec<String>,
    /// Modules in the center
    pub modules_center: Vec<String>,
    /// Modules on the right side (feature-detected)
    pub modules_right: Vec<WaybarModule>,
    /// Enable system tray
    pub tray: bool,
}

impl Default for WaybarConfig {
    fn default() -> Self {
        WaybarConfig {
            enabled: true,
            position: BarPosition::Top,
            height: 30,
            modules_left: vec!["sway/workspaces".to_string(), "sway/mode".to_string()],
            modules_center: vec!["clock".to_string()],
            modules_right: vec![],
            tray: true,
        }
    }
}
