/// Theme selection model - defines which theme is selected
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeSelection {
    /// Display name of the selected theme (e.g. "catppuccin-mocha")
    pub name: String,
    /// Source label ("built-in", "user", "system")
    pub source: String,
    /// Full path to the .env file for this theme
    #[serde(default)]
    pub path: String,
}

impl ThemeSelection {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        ThemeSelection {
            name: name.into(),
            source: source.into(),
            path: String::new(),
        }
    }

    pub fn with_path(
        name: impl Into<String>,
        source: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        ThemeSelection {
            name: name.into(),
            source: source.into(),
            path: path.into(),
        }
    }
}

/// Theme-specific overrides applied on top of the selected .env file.
/// None = use the value from the .env.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallpaper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_inner: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_outer: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waybar_opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_opacity: Option<f64>,
}
