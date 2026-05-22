/// Theme selection model - defines which theme is selected
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeSelection {
    /// Name of the selected theme
    pub name: String,
    /// Source of the theme (e.g., "system", "custom", etc.)
    pub source: String,
}

impl ThemeSelection {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        ThemeSelection {
            name: name.into(),
            source: source.into(),
        }
    }
}
