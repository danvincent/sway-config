/// Output/Display configuration settings
use serde::{Deserialize, Serialize};

/// Editable output/display settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputConfig {
    pub name: String,
    pub enabled: bool,
    pub resolution: Option<Resolution>,
    pub refresh_rate: Option<i32>,
    pub position: Position,
    pub scale: f64,
    pub transform: Transform,
}

/// Screen resolution (width x height in pixels)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

/// Output position on the virtual screen
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Display transform/rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Transform {
    #[default]
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl OutputConfig {
    /// Create OutputConfig from a detected Sway output
    pub fn from_sway(output: &crate::config::detect::SwayOutput) -> Self {
        let resolution = output.current_mode.as_ref().map(|mode| Resolution {
            width: mode.width,
            height: mode.height,
        });
        
        let refresh_rate = output.current_mode.as_ref().map(|mode| mode.refresh);
        
        let transform = match output.transform.as_str() {
            "90" => Transform::Rotate90,
            "180" => Transform::Rotate180,
            "270" => Transform::Rotate270,
            "flipped" => Transform::Flipped,
            "flipped-90" => Transform::Flipped90,
            "flipped-180" => Transform::Flipped180,
            "flipped-270" => Transform::Flipped270,
            _ => Transform::Normal,
        };
        
        OutputConfig {
            name: output.name.clone(),
            enabled: output.active,
            resolution,
            refresh_rate,
            position: Position {
                x: output.rect.x,
                y: output.rect.y,
            },
            scale: output.scale,
            transform,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_default() {
        assert_eq!(Transform::default(), Transform::Normal);
    }
    
    #[test]
    fn test_output_config_serialization() {
        let config = OutputConfig {
            name: "eDP-1".to_string(),
            enabled: true,
            resolution: Some(Resolution {
                width: 1920,
                height: 1080,
            }),
            refresh_rate: Some(60000),
            position: Position { x: 0, y: 0 },
            scale: 1.0,
            transform: Transform::Normal,
        };
        
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: OutputConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(config, deserialized);
    }
}
