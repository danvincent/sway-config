/// Sway session detection - higher-level wrapper around swaymsg
use serde::{Deserialize, Serialize};
use crate::config::swaymsg;

/// Represents a single output/display from Sway
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwayOutput {
    pub name: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub active: bool,
    pub dpms: bool,
    pub primary: bool,
    pub rect: Rect,
    pub current_mode: Option<Mode>,
    pub modes: Vec<Mode>,
    pub scale: f64,
    pub transform: String,
    pub focused: bool,
}

/// Rectangle representing output position and dimensions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Display mode (resolution and refresh rate)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

/// Represents a single input device from Sway
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwayInput {
    pub identifier: String,
    pub name: String,
    pub vendor: i32,
    pub product: i32,
    #[serde(rename = "type")]
    pub type_: String,
    /// Human-readable active layout name, e.g. "English (UK)". Use xkb_layouts_as_symbols
    /// for the actual XKB symbol string that should appear in sway config.
    pub xkb_active_layout_name: Option<String>,
    /// XKB layout symbols for each configured layout, e.g. ["gb", "us"].
    /// The first symbol corresponds to the current active layout.
    #[serde(default)]
    pub xkb_layouts_as_symbols: Vec<String>,
    pub libinput: Option<LibinputConfig>,
}

/// Libinput configuration for input devices
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LibinputConfig {
    pub send_events: Option<String>,
    pub tap: Option<String>,
    pub natural_scroll: Option<String>,
    pub dwt: Option<String>,
    pub accel_speed: Option<f64>,
    pub accel_profile: Option<String>,
    pub left_handed: Option<String>,
    pub middle_emulation: Option<String>,
}

/// Detect all outputs from the active Sway session
/// Returns empty vec if Sway is not running or detection fails
pub fn detect_outputs() -> Vec<SwayOutput> {
    swaymsg::get_outputs().unwrap_or_default()
}

/// Detect all inputs from the active Sway session
/// Returns empty vec if Sway is not running or detection fails
pub fn detect_inputs() -> Vec<SwayInput> {
    swaymsg::get_inputs().unwrap_or_default()
}

/// Filter inputs to find keyboards
pub fn detect_keyboards(inputs: &[SwayInput]) -> Vec<&SwayInput> {
    inputs
        .iter()
        .filter(|input| input.type_ == "keyboard")
        .collect()
}

/// Filter inputs to find touchpads
pub fn detect_touchpads(inputs: &[SwayInput]) -> Vec<&SwayInput> {
    inputs
        .iter()
        .filter(|input| {
            input.type_ == "touchpad"
                || input.type_ == "touch"
                || input.name.to_lowercase().contains("touchpad")
        })
        .collect()
}

/// Read the system keyboard layout from /etc/default/keyboard.
/// Returns `(layout, variant)` — layout defaults to "us", variant to "".
/// Used as the fallback when a keyboard device's `xkb_layouts_as_symbols` list is
/// empty (e.g. when running outside a Sway session or on hardware that doesn't
/// report XKB symbol info via swaymsg).
pub fn system_keyboard_layout() -> (String, String) {
    system_keyboard_layout_from_path("/etc/default/keyboard")
}

/// Inner implementation parameterised on path for testability.
pub fn system_keyboard_layout_from_path(path: &str) -> (String, String) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return ("us".to_string(), String::new()),
    };

    let mut layout = "us".to_string();
    let mut variant = String::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("XKBLAYOUT=") {
            layout = val.trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("XKBVARIANT=") {
            variant = val.trim_matches('"').to_string();
        }
    }

    (layout, variant)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rect_deserialize() {
        let json = r#"{"x":0,"y":0,"width":1920,"height":1080}"#;
        let rect: Rect = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 1920);
        assert_eq!(rect.height, 1080);
    }
    
    #[test]
    fn test_mode_deserialize() {
        let json = r#"{"width":1920,"height":1080,"refresh":60000}"#;
        let mode: Mode = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh, 60000);
    }
}
