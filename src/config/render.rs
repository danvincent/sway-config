/// Thin wrapper for rendering and applying configuration from AppState
use crate::config::apply::{apply, ApplyConfig, ApplyResult};
use crate::state::AppState;

/// Render and apply configuration from app state to user's config directory.
/// If `state.config_path()` is set (test mode), that path is used as the
/// base directory instead of the XDG default.
pub fn render_and_apply(state: &AppState) -> ApplyResult {
    let config_dir = state.config_path()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(dirs_config_home);
    apply(state.settings(), ApplyConfig::default(), &config_dir)
}

/// Render configuration without applying (dry run)
pub fn render_dry_run(state: &AppState) -> ApplyResult {
    let config_dir = state.config_path()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(dirs_config_home);
    apply(
        state.settings(),
        ApplyConfig {
            dry_run: true,
            reload_sway: false,
            restart_waybar: false,
        },
        &config_dir,
    )
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
