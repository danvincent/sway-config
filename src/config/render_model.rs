/// Intermediate render model - generated from settings, separate from editable state
use serde::{Deserialize, Serialize};
use crate::model::settings::Settings;
use crate::model::input::AccelProfile;

/// Complete render model with all configuration sections
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderModel {
    /// Rendered outputs
    pub outputs: Vec<RenderedOutput>,
    /// Rendered inputs (keyboards and touchpads)
    pub inputs: Vec<RenderedInput>,
    /// Rendered idle configuration (optional)
    pub idle: Option<RenderedIdle>,
    /// Rendered waybar configuration (optional)
    pub waybar: Option<RenderedWaybar>,
    /// Rendered autostart entries
    pub autostart: Vec<RenderedAutostart>,
    /// Theme name that will be applied
    pub theme_name: Option<String>,
    /// Theme source that will be used
    pub theme_source: Option<String>,
}

/// Rendered output with sway directive string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedOutput {
    /// Sway output directive
    pub sway_directive: String,
    /// Output name (identifier)
    pub name: String,
}

/// Rendered input (keyboard or touchpad) with sway directive
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedInput {
    /// Sway input directive block
    pub sway_directive: String,
    /// Input device identifier
    pub identifier: String,
}

/// Rendered idle configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedIdle {
    /// Full swayidle exec command line
    pub swayidle_exec: String,
}

/// Rendered waybar configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedWaybar {
    /// Waybar config JSON string
    pub config_json: String,
}

/// Rendered autostart entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedAutostart {
    /// Sway exec line
    pub exec_line: String,
    /// Description of the entry
    pub description: String,
}

impl RenderModel {
    /// Generate a render model from the current settings
    pub fn from_settings(settings: &Settings) -> Self {
        // Render outputs
        let outputs = settings
            .outputs
            .iter()
            .map(|output| {
                let sway_directive = if output.enabled {
                    let resolution_str = output
                        .resolution
                        .as_ref()
                        .map(|res| format!("{}x{}", res.width, res.height))
                        .unwrap_or_default();

                    let resolution_part = if resolution_str.is_empty() {
                        String::new()
                    } else {
                        format!("resolution {} ", resolution_str)
                    };

                    let transform_str = output.transform.to_sway_str();
                    let transform_part = if transform_str == "normal" {
                        String::new()
                    } else {
                        format!("transform {} ", transform_str)
                    };

                    format!(
                        "output {} {}position {} {} scale {} {}",
                        output.name,
                        resolution_part,
                        output.position.x,
                        output.position.y,
                        output.scale,
                        transform_part
                    )
                    .trim()
                    .to_string()
                } else {
                    format!("output {} disable", output.name)
                };

                RenderedOutput {
                    sway_directive,
                    name: output.name.clone(),
                }
            })
            .collect();

        // Render inputs
        let mut inputs = Vec::new();

        // Render keyboards
        for keyboard in &settings.keyboards {
            let sway_directive = format!(
                "input \"{}\" {{\n    xkb_layout {}\n    xkb_variant {}\n    xkb_options {}\n    repeat_delay {}\n    repeat_rate {}\n}}",
                keyboard.identifier,
                if keyboard.xkb_layout.is_empty() {
                    "us".to_string()
                } else {
                    keyboard.xkb_layout.clone()
                },
                if keyboard.xkb_variant.is_empty() {
                    "\"\"".to_string()
                } else {
                    format!("\"{}\"", keyboard.xkb_variant)
                },
                if keyboard.xkb_options.is_empty() {
                    "\"\"".to_string()
                } else {
                    format!("\"{}\"", keyboard.xkb_options)
                },
                keyboard.repeat_delay,
                keyboard.repeat_rate
            );

            inputs.push(RenderedInput {
                sway_directive,
                identifier: keyboard.identifier.clone(),
            });
        }

        // Render touchpads
        for touchpad in &settings.touchpads {
            let accel_profile_str = match touchpad.accel_profile {
                AccelProfile::Adaptive => "adaptive",
                AccelProfile::Flat => "flat",
            };

            let tap_str = if touchpad.tap_to_click { "enabled" } else { "disabled" };
            let scroll_str = if touchpad.natural_scroll { "enabled" } else { "disabled" };
            let dwt_str = if touchpad.dwt { "enabled" } else { "disabled" };
            let left_str = if touchpad.left_handed { "enabled" } else { "disabled" };
            let middle_str = if touchpad.middle_emulation { "enabled" } else { "disabled" };

            let sway_directive = format!(
                "input \"{}\" {{\n    tap {}\n    natural_scroll {}\n    dwt {}\n    accel_speed {}\n    accel_profile {}\n    left_handed {}\n    middle_emulation {}\n}}",
                touchpad.identifier,
                tap_str,
                scroll_str,
                dwt_str,
                touchpad.accel_speed,
                accel_profile_str,
                left_str,
                middle_str
            );

            inputs.push(RenderedInput {
                sway_directive,
                identifier: touchpad.identifier.clone(),
            });
        }

        // Render idle configuration
        let idle = if settings.idle.lock_timeout > 0 {
            let mut exec_parts = vec![
                format!("timeout {} '{}'", settings.idle.lock_timeout, settings.idle.lock_command),
            ];

            if settings.idle.before_sleep {
                exec_parts.push(format!("before-sleep '{}'", settings.idle.lock_command));
            }

            if settings.idle.screen_off_timeout > 0 {
                exec_parts.push(format!(
                    "timeout {} 'swaymsg \"output * dpms off\"' resume 'swaymsg \"output * dpms on\"'",
                    settings.idle.screen_off_timeout
                ));
            }

            Some(RenderedIdle {
                swayidle_exec: format!("exec swayidle -w {}", exec_parts.join(" ")),
            })
        } else {
            None
        };

        // Render waybar configuration
        let waybar = if settings.waybar.enabled {
            let modules_right: Vec<String> = settings
                .waybar
                .modules_right
                .iter()
                .filter(|m| m.enabled)
                .map(|m| format!("\"{}\"", m.name))
                .collect();

            let position_str = match settings.waybar.position {
                crate::model::waybar::BarPosition::Top => "top",
                crate::model::waybar::BarPosition::Bottom => "bottom",
                crate::model::waybar::BarPosition::Left => "left",
                crate::model::waybar::BarPosition::Right => "right",
            };

            let modules_left: Vec<String> = settings
                .waybar
                .modules_left
                .iter()
                .map(|m| format!("\"{}\"", m))
                .collect();

            let modules_center: Vec<String> = settings
                .waybar
                .modules_center
                .iter()
                .map(|m| format!("\"{}\"", m))
                .collect();

            let json_obj = serde_json::json!({
                "position": position_str,
                "height": settings.waybar.height,
                "modules-left": modules_left.iter().map(|m| m.trim_matches('"')).collect::<Vec<_>>(),
                "modules-center": modules_center.iter().map(|m| m.trim_matches('"')).collect::<Vec<_>>(),
                "modules-right": modules_right.iter().map(|m| m.trim_matches('"')).collect::<Vec<_>>(),
            });

            // "tray" as a top-level key must be an object — omit it entirely;
            // the tray module is already included in modules-right when enabled.
            let config_json = serde_json::to_string_pretty(&json_obj)
                .unwrap_or_else(|_| "{}".to_string());

            Some(RenderedWaybar { config_json })
        } else {
            None
        };

        // Render autostart entries
        let autostart = settings
            .autostart
            .entries
            .iter()
            .filter(|entry| entry.enabled)
            .map(|entry| RenderedAutostart {
                exec_line: format!("# {}\nexec {}", entry.description, entry.command),
                description: entry.description.clone(),
            })
            .collect();

        // Theme
        let (theme_name, theme_source) = settings
            .theme
            .as_ref()
            .map(|t| (Some(t.name.clone()), Some(t.source.clone())))
            .unwrap_or((None, None));

        RenderModel {
            outputs,
            inputs,
            idle,
            waybar,
            autostart,
            theme_name,
            theme_source,
        }
    }

    /// Render outputs.conf content
    pub fn to_sway_outputs_conf(&self) -> String {
        self.outputs
            .iter()
            .map(|output| output.sway_directive.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render inputs.conf content
    pub fn to_sway_inputs_conf(&self) -> String {
        self.inputs
            .iter()
            .map(|input| input.sway_directive.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Render idle.conf content
    pub fn to_sway_idle_conf(&self) -> String {
        self.idle
            .as_ref()
            .map(|idle| idle.swayidle_exec.clone())
            .unwrap_or_default()
    }

    /// Render waybar config.json content
    pub fn to_waybar_config_json(&self) -> String {
        self.waybar
            .as_ref()
            .map(|waybar| waybar.config_json.clone())
            .unwrap_or_default()
    }

    /// Render autostart.conf content
    pub fn to_autostart_conf(&self) -> String {
        self.autostart
            .iter()
            .map(|entry| entry.exec_line.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::theme::ThemeSelection;

    #[test]
    fn test_render_model_from_settings_no_theme() {
        let settings = Settings::default();
        let model = RenderModel::from_settings(&settings);
        assert!(model.theme_name.is_none());
        assert!(model.theme_source.is_none());
        assert!(model.outputs.is_empty());
    }

    #[test]
    fn test_render_model_from_settings_with_theme() {
        let mut settings = Settings::default();
        settings.theme = Some(ThemeSelection::new("dark", "system"));
        let model = RenderModel::from_settings(&settings);
        assert_eq!(model.theme_name, Some("dark".to_string()));
        assert_eq!(model.theme_source, Some("system".to_string()));
    }

    #[test]
    fn test_render_model_serialization() {
        let model = RenderModel {
            outputs: vec![],
            inputs: vec![],
            idle: None,
            waybar: None,
            autostart: vec![],
            theme_name: Some("light".to_string()),
            theme_source: Some("custom".to_string()),
        };

        let json = serde_json::to_string(&model).expect("Failed to serialize");
        let deserialized: RenderModel =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(model, deserialized);
    }

    #[test]
    fn test_render_output_includes_resolution() {
        use crate::model::output::{OutputConfig, Position, Resolution, Transform};
        let mut settings = Settings::default();
        settings.outputs = vec![OutputConfig {
            name: "HDMI-1".to_string(),
            enabled: true,
            resolution: Some(Resolution { width: 1920, height: 1080 }),
            refresh_rate: Some(144000),
            position: Position { x: 0, y: 0 },
            scale: 1.0,
            transform: Transform::Normal,
        }];
        let model = RenderModel::from_settings(&settings);
        let directive = &model.outputs[0].sway_directive;
        assert!(
            directive.contains("1920x1080"),
            "resolution must appear in directive, got: {directive}"
        );
    }

    #[test]
    fn test_render_different_resolutions_produce_different_directives() {
        use crate::model::output::{OutputConfig, Position, Resolution, Transform};

        let make_settings = |w: i32, h: i32| {
            let mut settings = Settings::default();
            settings.outputs = vec![OutputConfig {
                name: "DP-1".to_string(),
                enabled: true,
                resolution: Some(Resolution { width: w, height: h }),
                refresh_rate: Some(60000),
                position: Position { x: 0, y: 0 },
                scale: 1.0,
                transform: Transform::Normal,
            }];
            RenderModel::from_settings(&settings).outputs[0].sway_directive.clone()
        };

        let d1080 = make_settings(1920, 1080);
        let d1440 = make_settings(2560, 1440);
        assert_ne!(d1080, d1440, "different resolutions must produce different directives");
        assert!(d1080.contains("1920x1080"), "1080p directive: {d1080}");
        assert!(d1440.contains("2560x1440"), "1440p directive: {d1440}");
    }

    #[test]
    fn test_render_output_no_at_sign_in_directive() {
        use crate::model::output::{OutputConfig, Position, Resolution, Transform};
        let mut settings = Settings::default();
        settings.outputs = vec![OutputConfig {
            name: "eDP-1".to_string(),
            enabled: true,
            resolution: Some(Resolution { width: 1920, height: 1080 }),
            refresh_rate: None,
            position: Position { x: 0, y: 0 },
            scale: 1.0,
            transform: Transform::Normal,
        }];
        let model = RenderModel::from_settings(&settings);
        let directive = &model.outputs[0].sway_directive;
        assert!(
            !directive.contains('@'),
            "no '@' expected in output directive, got: {directive}"
        );
    }

    #[test]
    fn test_render_keyboard_layout_symbol_in_directive() {
        use crate::model::input::KeyboardConfig;
        let mut settings = Settings::default();
        settings.keyboards = vec![KeyboardConfig {
            identifier: "1:1:AT_Translated_Set_2_keyboard".to_string(),
            xkb_layout: "gb".to_string(),
            xkb_variant: String::new(),
            xkb_options: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
        }];
        let model = RenderModel::from_settings(&settings);
        let directive = &model.inputs[0].sway_directive;
        assert!(
            directive.contains("xkb_layout gb"),
            "directive must contain layout symbol 'gb', got: {directive}"
        );
        assert!(
            !directive.contains("English"),
            "directive must not contain display name, got: {directive}"
        );
    }

    #[test]
    fn test_render_keyboard_different_layouts_produce_different_directives() {
        use crate::model::input::KeyboardConfig;

        let make = |layout: &str| {
            let mut settings = Settings::default();
            settings.keyboards = vec![KeyboardConfig {
                identifier: "kb".to_string(),
                xkb_layout: layout.to_string(),
                xkb_variant: String::new(),
                xkb_options: String::new(),
                repeat_delay: 600,
                repeat_rate: 25,
            }];
            RenderModel::from_settings(&settings).inputs[0].sway_directive.clone()
        };

        let gb = make("gb");
        let us = make("us");
        let de = make("de");
        assert_ne!(gb, us);
        assert_ne!(gb, de);
        assert!(gb.contains("xkb_layout gb"), "gb directive: {gb}");
        assert!(us.contains("xkb_layout us"), "us directive: {us}");
    }
}

