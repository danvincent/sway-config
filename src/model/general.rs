/// General sway configuration settings
use serde::{Deserialize, Serialize};

/// General sway environment settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default terminal emulator command (e.g. "alacritty", "foot", "kitty")
    /// This sets $term in sway config. Default: empty string means auto-detect.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub terminal: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            terminal: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_config_default_terminal_is_empty() {
        let config = GeneralConfig::default();
        assert!(config.terminal.is_empty());
    }

    #[test]
    fn test_general_config_terminal_roundtrip() {
        let mut config = GeneralConfig::default();
        config.terminal = "alacritty".to_string();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let reloaded: GeneralConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(reloaded.terminal, "alacritty");
    }

    #[test]
    fn test_general_config_empty_terminal_not_serialized() {
        let config = GeneralConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(!toml_str.contains("terminal"),
            "Empty terminal should not appear in serialized TOML");
    }
}
