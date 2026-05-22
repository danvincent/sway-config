/// Idle configuration - screensaver and lock behavior
use serde::{Deserialize, Serialize};

/// Configuration for idle behavior (screensaver, lock, sleep)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdleConfig {
    /// Lock screen timeout in seconds. 0 = disabled. Default: 300 (5 minutes)
    #[serde(default = "default_lock_timeout")]
    pub lock_timeout: u32,
    /// Screen off timeout in seconds. 0 = disabled. Default: 0
    #[serde(default)]
    pub screen_off_timeout: u32,
    /// Command to run to lock the screen. Default: "swaylock"
    #[serde(default = "default_lock_command")]
    pub lock_command: String,
    /// Lock screen before sleep. Default: true
    #[serde(default = "default_before_sleep")]
    pub before_sleep: bool,
}

fn default_lock_timeout() -> u32 {
    300
}

fn default_lock_command() -> String {
    "swaylock".to_string()
}

fn default_before_sleep() -> bool {
    true
}

impl Default for IdleConfig {
    fn default() -> Self {
        IdleConfig {
            lock_timeout: 300,
            screen_off_timeout: 0,
            lock_command: "swaylock".to_string(),
            before_sleep: true,
        }
    }
}
