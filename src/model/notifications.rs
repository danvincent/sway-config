/// Notifications configuration - notification daemon settings
use serde::{Deserialize, Serialize};

/// Position of notifications on screen
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifPosition {
    /// Top right corner
    #[default]
    TopRight,
    /// Top left corner
    TopLeft,
    /// Top center
    TopCenter,
    /// Bottom right corner
    BottomRight,
    /// Bottom left corner
    BottomLeft,
    /// Bottom center
    BottomCenter,
    /// Center of screen
    Center,
}

/// Configuration for the notification daemon
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    /// Notification timeout in milliseconds. Default: 5000
    pub timeout_ms: u32,
    /// Maximum visible notifications. Default: 5
    pub max_visible: u32,
    /// Position of notifications on screen
    pub position: NotifPosition,
    /// Follow keyboard focus (some daemons support this). Default: false
    pub follow_focus: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        NotificationsConfig {
            timeout_ms: 5000,
            max_visible: 5,
            position: NotifPosition::TopRight,
            follow_focus: false,
        }
    }
}
