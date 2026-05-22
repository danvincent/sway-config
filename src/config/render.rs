/// Thin wrapper for rendering and applying configuration from AppState
use crate::config::apply::{apply, apply_theme, ApplyConfig, ApplyResult};
use crate::state::AppState;

/// Render and apply configuration from app state to user's config directory.
/// If `state.config_path()` is set (test mode), that path is used as the
/// base directory instead of the XDG default.
pub fn render_and_apply(state: &AppState) -> ApplyResult {
    let config_dir = state
        .config_path()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(dirs_config_home);
    let mut result = apply(state.settings(), ApplyConfig::default(), &config_dir);

    if result.success {
        if let Some(theme) = state.settings().theme.clone() {
            let theme_result = apply_theme(
                &theme,
                &state.settings().theme_overrides,
                &config_dir,
                false,
            );
            result.success = result.success && theme_result.success;
            result.files_written.extend(theme_result.files_written);
            result.errors.extend(theme_result.errors);
            result.sway_reloaded = result.sway_reloaded || theme_result.sway_reloaded;
            result.waybar_restarted = result.waybar_restarted || theme_result.waybar_restarted;
        }
    }

    result
}

/// Render configuration without applying (dry run)
pub fn render_dry_run(state: &AppState) -> ApplyResult {
    let config_dir = state
        .config_path()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(dirs_config_home);
    let mut result = apply(
        state.settings(),
        ApplyConfig {
            dry_run: true,
            reload_sway: false,
            restart_waybar: false,
        },
        &config_dir,
    );

    if result.success {
        if let Some(theme) = state.settings().theme.clone() {
            let theme_result =
                apply_theme(&theme, &state.settings().theme_overrides, &config_dir, true);
            result.success = result.success && theme_result.success;
            result.files_written.extend(theme_result.files_written);
            result.errors.extend(theme_result.errors);
        }
    }

    result
}

/// Get the XDG_CONFIG_HOME directory, defaulting to ~/.config
fn dirs_config_home() -> std::path::PathBuf {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(xdg_config)
    } else {
        let mut path = dirs_home();
        path.push(".config");
        path
    }
}

/// Get home directory
fn dirs_home() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
    } else {
        std::path::PathBuf::from("/root")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirs_config_home_with_env() {
        // Just verify it returns a PathBuf without panicking
        let _path = dirs_config_home();
    }
}
