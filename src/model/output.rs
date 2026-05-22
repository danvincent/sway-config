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

impl Transform {
    /// Convert transform to sway-compatible string format
    pub fn to_sway_str(&self) -> &'static str {
        match self {
            Transform::Normal => "normal",
            Transform::Rotate90 => "90",
            Transform::Rotate180 => "180",
            Transform::Rotate270 => "270",
            Transform::Flipped => "flipped",
            Transform::Flipped90 => "flipped-90",
            Transform::Flipped180 => "flipped-180",
            Transform::Flipped270 => "flipped-270",
        }
    }
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
    fn test_transform_to_sway_str_normal() {
        assert_eq!(Transform::Normal.to_sway_str(), "normal");
    }
    
    #[test]
    fn test_transform_to_sway_str_rotate90() {
        assert_eq!(Transform::Rotate90.to_sway_str(), "90");
    }
    
    #[test]
    fn test_transform_to_sway_str_rotate180() {
        assert_eq!(Transform::Rotate180.to_sway_str(), "180");
    }
    
    #[test]
    fn test_transform_to_sway_str_rotate270() {
        assert_eq!(Transform::Rotate270.to_sway_str(), "270");
    }
    
    #[test]
    fn test_transform_to_sway_str_flipped() {
        assert_eq!(Transform::Flipped.to_sway_str(), "flipped");
    }
    
    #[test]
    fn test_transform_to_sway_str_flipped90() {
        assert_eq!(Transform::Flipped90.to_sway_str(), "flipped-90");
    }
    
    #[test]
    fn test_transform_to_sway_str_flipped180() {
        assert_eq!(Transform::Flipped180.to_sway_str(), "flipped-180");
    }
    
    #[test]
    fn test_transform_to_sway_str_flipped270() {
        assert_eq!(Transform::Flipped270.to_sway_str(), "flipped-270");
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
