/// Input device configuration settings (keyboard, touchpad)
use serde::{Deserialize, Serialize};

/// Keyboard configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub identifier: String,
    #[serde(default = "default_layout")]
    pub xkb_layout: String,
    #[serde(default)]
    pub xkb_variant: String,
    #[serde(default)]
    pub xkb_options: String,
    #[serde(default = "default_repeat_delay")]
    pub repeat_delay: i32,
    #[serde(default = "default_repeat_rate")]
    pub repeat_rate: i32,
}

/// Touchpad configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TouchpadConfig {
    pub identifier: String,
    #[serde(default)]
    pub tap_to_click: bool,
    #[serde(default)]
    pub natural_scroll: bool,
    #[serde(default)]
    pub dwt: bool,
    #[serde(default)]
    pub accel_speed: f64,
    #[serde(default)]
    pub accel_profile: AccelProfile,
    #[serde(default)]
    pub left_handed: bool,
    #[serde(default)]
    pub middle_emulation: bool,
}

/// Acceleration profile for input devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AccelProfile {
    #[default]
    Adaptive,
    Flat,
}

// Default value functions for serde
fn default_layout() -> String {
    "us".to_string()
}

fn default_repeat_delay() -> i32 {
    600
}

fn default_repeat_rate() -> i32 {
    25
}

impl KeyboardConfig {
    /// Create KeyboardConfig from a detected Sway input (keyboard type).
    /// The xkb_layout is populated from:
    ///   1. `xkb_layouts_as_symbols[0]` from swaymsg — the XKB symbol (e.g. "gb", "us")
    ///      that corresponds to the active layout; this is what sway config expects.
    ///   2. `/etc/default/keyboard` (XKBLAYOUT= line) as fallback when swaymsg doesn't
    ///      provide symbols (e.g. when running outside a Sway session).
    ///   3. "us" if neither source is available.
    pub fn from_sway(input: &crate::config::detect::SwayInput) -> Self {
        let (sys_layout, sys_variant) =
            crate::config::detect::system_keyboard_layout();

        let xkb_layout = input
            .xkb_layouts_as_symbols
            .first()
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or(sys_layout);

        KeyboardConfig {
            identifier: input.identifier.clone(),
            xkb_layout,
            xkb_variant: sys_variant,
            xkb_options: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
        }
    }
}

impl TouchpadConfig {
    /// Create TouchpadConfig from a detected Sway input (touchpad type)
    pub fn from_sway(input: &crate::config::detect::SwayInput) -> Self {
        let libinput = input.libinput.as_ref();
        
        let tap_to_click = libinput
            .and_then(|cfg| cfg.tap.as_ref())
            .map(|s| s == "enabled")
            .unwrap_or(false);
        
        let natural_scroll = libinput
            .and_then(|cfg| cfg.natural_scroll.as_ref())
            .map(|s| s == "enabled")
            .unwrap_or(false);
        
        let dwt = libinput
            .and_then(|cfg| cfg.dwt.as_ref())
            .map(|s| s == "enabled")
            .unwrap_or(false);
        
        let accel_speed = libinput
            .and_then(|cfg| cfg.accel_speed)
            .unwrap_or(0.0);
        
        let accel_profile = libinput
            .and_then(|cfg| cfg.accel_profile.as_ref())
            .map(|s| {
                if s == "flat" {
                    AccelProfile::Flat
                } else {
                    AccelProfile::Adaptive
                }
            })
            .unwrap_or(AccelProfile::Adaptive);
        
        let left_handed = libinput
            .and_then(|cfg| cfg.left_handed.as_ref())
            .map(|s| s == "enabled")
            .unwrap_or(false);
        
        let middle_emulation = libinput
            .and_then(|cfg| cfg.middle_emulation.as_ref())
            .map(|s| s == "enabled")
            .unwrap_or(false);
        
        TouchpadConfig {
            identifier: input.identifier.clone(),
            tap_to_click,
            natural_scroll,
            dwt,
            accel_speed,
            accel_profile,
            left_handed,
            middle_emulation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyboard_config_defaults() {
        let config = KeyboardConfig {
            identifier: "test".to_string(),
            xkb_layout: default_layout(),
            xkb_variant: String::new(),
            xkb_options: String::new(),
            repeat_delay: default_repeat_delay(),
            repeat_rate: default_repeat_rate(),
        };
        
        assert_eq!(config.xkb_layout, "us");
        assert_eq!(config.repeat_delay, 600);
        assert_eq!(config.repeat_rate, 25);
    }
    
    #[test]
    fn test_accel_profile_default() {
        assert_eq!(AccelProfile::default(), AccelProfile::Adaptive);
    }
    
    #[test]
    fn test_touchpad_config_serialization() {
        let config = TouchpadConfig {
            identifier: "test".to_string(),
            tap_to_click: true,
            natural_scroll: false,
            dwt: true,
            accel_speed: 0.5,
            accel_profile: AccelProfile::Adaptive,
            left_handed: false,
            middle_emulation: false,
        };
        
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: TouchpadConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(config, deserialized);
    }
}
