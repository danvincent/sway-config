/// Apply configuration to system - writes files and applies changes
use crate::config::render_model::RenderModel;
use crate::model::settings::Settings;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration options for applying settings
#[derive(Debug, Clone, Copy)]
pub struct ApplyConfig {
    /// If true, render but don't write files or reload
    pub dry_run: bool,
    /// If true, run swaymsg reload after writing files
    pub reload_sway: bool,
    /// If true, restart waybar after writing config
    pub restart_waybar: bool,
}

impl Default for ApplyConfig {
    fn default() -> Self {
        ApplyConfig {
            dry_run: false,
            reload_sway: true,
            restart_waybar: true,
        }
    }
}

/// Result of applying configuration
#[derive(Debug, Clone)]
pub struct ApplyResult {
    /// Whether the apply operation succeeded
    pub success: bool,
    /// List of files that were written (or would be written in dry-run)
    pub files_written: Vec<String>,
    /// List of errors encountered
    pub errors: Vec<String>,
    /// Whether sway was reloaded
    pub sway_reloaded: bool,
    /// Whether waybar was restarted
    pub waybar_restarted: bool,
}

/// Paths for all output config files
#[derive(Debug)]
struct OutputPaths {
    outputs_conf: PathBuf,
    inputs_conf: PathBuf,
    idle_conf: PathBuf,
    waybar_config: PathBuf,
    #[allow(dead_code)]
    waybar_style: PathBuf,
    autostart_conf: PathBuf,
}

impl OutputPaths {
    fn new(base: &Path) -> Self {
        OutputPaths {
            outputs_conf: base.join("sway/config.d/outputs.conf"),
            inputs_conf: base.join("sway/config.d/inputs.conf"),
            idle_conf: base.join("sway/config.d/idle.conf"),
            waybar_config: base.join("waybar/config.jsonc"),
            waybar_style: base.join("waybar/style.css"),
            autostart_conf: base.join("sway/config.d/autostart.conf"),
        }
    }
}

/// Write file, creating parent directories if needed
fn write_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

/// Apply settings to system files
///
/// # Arguments
/// * `settings` - The current Settings to apply
/// * `config` - Apply configuration options
/// * `base_path` - Base path for config files (typically ~/.config)
pub fn apply(settings: &Settings, config: ApplyConfig, base_path: &Path) -> ApplyResult {
    let mut result = ApplyResult {
        success: true,
        files_written: Vec::new(),
        errors: Vec::new(),
        sway_reloaded: false,
        waybar_restarted: false,
    };

    // Build render model
    let model = RenderModel::from_settings(settings);
    let paths = OutputPaths::new(base_path);

    // Generate all config files
    let outputs_conf = model.to_sway_outputs_conf();
    let inputs_conf = model.to_sway_inputs_conf();
    let idle_conf = model.to_sway_idle_conf();
    let waybar_json = model.to_waybar_config_json();
    let autostart_conf = model.to_autostart_conf();

    // Check if waybar is actually enabled
    let has_waybar = !waybar_json.is_empty();
    
    // For waybar, always write config.json, but use {} if disabled
    let waybar_json_output = if waybar_json.is_empty() {
        "{}".to_string()
    } else {
        waybar_json
    };

    // List of (path, content) pairs to write - always write all managed files
    // Outputs and inputs always have content (even if empty), but idle and autostart
    // may be empty when disabled. We write them anyway to clear stale files.
    let files_to_write: Vec<(PathBuf, String)> = vec![
        (paths.outputs_conf.clone(), outputs_conf),
        (paths.inputs_conf.clone(), inputs_conf),
        (paths.idle_conf.clone(), idle_conf),
        (paths.autostart_conf.clone(), autostart_conf),
        (paths.waybar_config.clone(), waybar_json_output),
    ];

    // Track files that would be written
    for (path, _) in &files_to_write {
        result.files_written.push(path.to_string_lossy().to_string());
    }

    // If dry run, return here without writing
    if config.dry_run {
        return result;
    }

    // Write all managed files
    for (path, content) in files_to_write {
        if let Err(e) = write_file(&path, &content) {
            result.success = false;
            result.errors.push(format!(
                "Failed to write {}: {}",
                path.display(),
                e
            ));
        }
    }

    // Reload sway if configured and successful so far
    if config.reload_sway && result.success {
        result.sway_reloaded = reload_sway();
        if !result.sway_reloaded {
            result.errors.push("Failed to reload sway".to_string());
        }
    }

    // Restart waybar if configured and successful so far
    if config.restart_waybar && result.success && has_waybar {
        result.waybar_restarted = restart_waybar();
        if !result.waybar_restarted {
            result.errors.push("Failed to restart waybar (may not be running)".to_string());
        }
    }

    result
}

/// Reload sway configuration
fn reload_sway() -> bool {
    use std::process::Command;

    let output = Command::new("swaymsg")
        .arg("reload")
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

/// Restart waybar — uses systemd reload if available, otherwise kill by PID + swaymsg exec.
fn restart_waybar() -> bool {
    use std::process::Command;

    // Try systemd reload first (ExecReload=kill -SIGUSR2 $MAINPID)
    let reloaded = Command::new("systemctl")
        .args(["--user", "reload", "waybar"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if reloaded {
        return true;
    }

    // Kill any running waybar by PID, then relaunch via swaymsg (inherits Wayland env)
    if let Ok(output) = Command::new("pgrep").arg("waybar").output() {
        let pids = String::from_utf8_lossy(&output.stdout);
        for pid_str in pids.split_whitespace() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                let _ = Command::new("kill").arg(pid.to_string()).output();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    Command::new("swaymsg")
        .args(["exec", "waybar"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_config_default() {
        let config = ApplyConfig::default();
        assert!(!config.dry_run);
        assert!(config.reload_sway);
        assert!(config.restart_waybar);
    }

    #[test]
    fn test_apply_result_creation() {
        let result = ApplyResult {
            success: true,
            files_written: vec![],
            errors: vec![],
            sway_reloaded: false,
            waybar_restarted: false,
        };

        assert!(result.success);
    }
}
