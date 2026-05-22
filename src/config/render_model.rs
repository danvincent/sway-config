/// Intermediate render model - generated from settings, separate from editable state
use serde::{Deserialize, Serialize};
use crate::model::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderModel {
    /// Theme name that will be applied
    pub theme_name: Option<String>,
    /// Theme source that will be used
    pub theme_source: Option<String>,
}

impl RenderModel {
    /// Generate a render model from the current settings
    pub fn from_settings(settings: &Settings) -> Self {
        let (theme_name, theme_source) = settings
            .theme
            .as_ref()
            .map(|t| (Some(t.name.clone()), Some(t.source.clone())))
            .unwrap_or((None, None));

        RenderModel {
            theme_name,
            theme_source,
        }
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
            theme_name: Some("light".to_string()),
            theme_source: Some("custom".to_string()),
        };

        let json = serde_json::to_string(&model).expect("Failed to serialize");
        let deserialized: RenderModel =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(model, deserialized);
    }
}
