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
        result
            .files_written
            .push(path.to_string_lossy().to_string());
    }

    // If dry run, return here without writing
    if config.dry_run {
        return result;
    }

    // Write all managed files
    for (path, content) in files_to_write {
        if let Err(e) = write_file(&path, &content) {
            result.success = false;
            result
                .errors
                .push(format!("Failed to write {}: {}", path.display(), e));
        }
    }

    // Reload sway if configured and successful so far
    if config.reload_sway && result.success {
        result.sway_reloaded = reload_sway();
        if !result.sway_reloaded {
            result.errors.push("Failed to reload sway".to_string());
        }
    }

    // Sync waybar process state if configured and successful so far:
    // - enabled => restart/reload it
    // - disabled => stop any running instance
    if config.restart_waybar && result.success {
        if has_waybar {
            result.waybar_restarted = restart_waybar();
            if !result.waybar_restarted {
                result
                    .errors
                    .push("Failed to restart waybar (may not be running)".to_string());
            }
        } else {
            result.waybar_restarted = stop_waybar();
            if !result.waybar_restarted {
                result.errors.push("Failed to stop waybar".to_string());
            }
        }
    }

    result
}

/// Reload sway configuration
fn reload_sway() -> bool {
    use std::process::Command;

    let output = Command::new("swaymsg").arg("reload").output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

/// Restart/reload waybar safely.
/// Tries user-service reload first; otherwise sends SIGUSR2 to running waybar.
/// Only launches a new instance when no waybar process is running.
fn restart_waybar() -> bool {
    use std::process::Command;

    // Try systemd reload first only when the user service is active.
    // This avoids noisy "waybar.service is not active, cannot reload." stderr output.
    let service_active = Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "waybar.service"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let reloaded = if service_active {
        Command::new("systemctl")
            .args(["--user", "reload", "waybar.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    };

    if reloaded {
        return true;
    }

    // Prefer in-place reload for non-systemd runs.
    if let Ok(output) = Command::new("pgrep").arg("waybar").output() {
        let pids = String::from_utf8_lossy(&output.stdout);
        if !pids.trim().is_empty() {
            let mut any_ok = false;
            for pid_str in pids.split_whitespace() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let ok = Command::new("kill")
                        .args(["-USR2", &pid.to_string()])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    any_ok = any_ok || ok;
                }
            }
            if any_ok {
                return true;
            }
        }
    }

    // No running process (or signal failed): launch via swaymsg so env is correct.
    Command::new("swaymsg")
        .args(["exec", "waybar"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Stop waybar if it is running. Returns true when no running instance remains.
fn stop_waybar() -> bool {
    use std::process::Command;

    let service_active = Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "waybar.service"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if service_active {
        let stopped = Command::new("systemctl")
            .args(["--user", "stop", "waybar.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if stopped {
            return true;
        }
    }

    if let Ok(output) = Command::new("pgrep").arg("waybar").output() {
        let pids = String::from_utf8_lossy(&output.stdout);
        if pids.trim().is_empty() {
            return true;
        }
        for pid_str in pids.split_whitespace() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                let _ = Command::new("kill").arg(pid.to_string()).output();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        let still_running = Command::new("pgrep")
            .arg("waybar")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        return !still_running;
    }

    true
}

/// Parse a theme .env file into a variable map.
/// Lines of the form KEY="value" or KEY=value are returned as HashMap entries.
pub fn parse_env_file(content: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim().to_string();
            let val = val.trim().trim_matches('"').to_string();
            if !key.is_empty() {
                map.insert(key, val);
            }
        }
    }
    map
}

/// Substitute `${TOKEN}` placeholders in a template string using the provided map.
pub fn envsubst(template: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, val) in vars {
        result = result.replace(&format!("${{{}}}", key), val);
    }
    result
}

/// Find the path to a theme `.env` file by name, scanning standard directories.
/// Used as a fallback when the stored `ThemeSelection.path` is empty.
fn find_theme_path_by_name(name: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let candidates = [
        format!("{}/.config/sway/themes/{}.env", home, name),
        format!("{}/source/SwayConfig/themes/{}.env", home, name),
    ];
    candidates
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
}

/// Apply a theme — writes sway colors, waybar style.css, GTK/Qt settings, wallpaper.
///
/// The theme `.env` file is parsed; overrides from `ThemeOverrides` are applied.
/// Set `dry_run` to true to skip all writes and reloads.
pub fn apply_theme(
    selection: &crate::model::theme::ThemeSelection,
    overrides: &crate::model::theme::ThemeOverrides,
    base_path: &Path,
    dry_run: bool,
) -> ApplyResult {
    let mut result = ApplyResult {
        success: true,
        files_written: Vec::new(),
        errors: Vec::new(),
        sway_reloaded: false,
        waybar_restarted: false,
    };

    // Resolve path — may be empty if loaded from settings saved before the path field was added.
    // Fall back to scanning the standard theme directories by name.
    let resolved_path: String = if !selection.path.is_empty() {
        selection.path.clone()
    } else {
        find_theme_path_by_name(&selection.name).unwrap_or_default()
    };

    // Parse .env file
    let env_content = match fs::read_to_string(&resolved_path) {
        Ok(c) => c,
        Err(e) => {
            result.success = false;
            result
                .errors
                .push(format!("Cannot read theme file {}: {}", resolved_path, e));
            return result;
        }
    };

    let mut vars = parse_env_file(&env_content);

    // Apply overrides
    if let Some(ref wp) = overrides.wallpaper {
        vars.insert("WALLPAPER_PATH".into(), wp.clone());
    }
    if let Some(ref ff) = overrides.font_family {
        vars.insert("FONT_FAMILY".into(), ff.clone());
    }
    if let Some(fs_) = overrides.font_size {
        vars.insert("FONT_SIZE".into(), fs_.to_string());
    }
    if let Some(gi) = overrides.gap_inner {
        vars.insert("GAP_INNER".into(), gi.to_string());
    }
    if let Some(go) = overrides.gap_outer {
        vars.insert("GAP_OUTER".into(), go.to_string());
    }
    if let Some(bw) = overrides.border_width {
        vars.insert("BORDER_WIDTH".into(), bw.to_string());
    }
    if let Some(wo) = overrides.waybar_opacity {
        vars.insert("WAYBAR_OPACITY".into(), format!("{:.2}", wo));
    }
    if let Some(to) = overrides.terminal_opacity {
        vars.insert("TERMINAL_OPACITY".into(), format!("{:.2}", to));
    }

    if dry_run {
        result
            .files_written
            .push("(dry-run) sway colors, waybar style, gtk/qt settings".into());
        return result;
    }

    // Write ~/.config/sway-theme pointer
    let sway_theme_ptr = dirs_or_home("sway-theme", base_path);
    if let Err(e) = write_file(&sway_theme_ptr, &format!("{}\n", selection.path)) {
        result
            .errors
            .push(format!("Failed to write sway-theme pointer: {}", e));
    } else {
        result
            .files_written
            .push(sway_theme_ptr.to_string_lossy().into());
    }

    // Write sway colors config
    let colors_conf = build_sway_colors_conf(&vars);
    let colors_path = base_path.join("sway/config.d/colors.conf");
    match write_file(&colors_path, &colors_conf) {
        Ok(_) => result
            .files_written
            .push(colors_path.to_string_lossy().into()),
        Err(e) => result
            .errors
            .push(format!("Failed to write colors.conf: {}", e)),
    }

    // Write waybar style.css from bundled template
    let waybar_css = envsubst(WAYBAR_STYLE_TEMPLATE, &vars);
    let style_path = base_path.join("waybar/style.css");
    match write_file(&style_path, &waybar_css) {
        Ok(_) => result
            .files_written
            .push(style_path.to_string_lossy().into()),
        Err(e) => result
            .errors
            .push(format!("Failed to write waybar style.css: {}", e)),
    }

    // Write GTK settings
    let gtk_settings = build_gtk_settings(&vars);
    for ver in &["gtk-3.0", "gtk-4.0"] {
        let gtk_path = base_path.join(format!("{}/settings.ini", ver));
        match write_file(&gtk_path, &gtk_settings) {
            Ok(_) => result.files_written.push(gtk_path.to_string_lossy().into()),
            Err(e) => result
                .errors
                .push(format!("Failed to write {}/settings.ini: {}", ver, e)),
        }
    }

    // Write qt5ct/qt6ct settings so Qt apps can follow icon/font and colors.
    let qt_colors = build_qtct_color_scheme(&vars);
    for dir in &["qt5ct", "qt6ct"] {
        let colors_path = base_path.join(format!("{}/colors/sway-config.conf", dir));
        match write_file(&colors_path, &qt_colors) {
            Ok(_) => result
                .files_written
                .push(colors_path.to_string_lossy().into()),
            Err(e) => result.errors.push(format!(
                "Failed to write {}/colors/sway-config.conf: {}",
                dir, e
            )),
        }

        let qt_settings = build_qtct_settings(&vars, &colors_path);
        let qt_path = base_path.join(format!("{}/{}.conf", dir, dir));
        match write_file(&qt_path, &qt_settings) {
            Ok(_) => result.files_written.push(qt_path.to_string_lossy().into()),
            Err(e) => result
                .errors
                .push(format!("Failed to write {}/{}.conf: {}", dir, dir, e)),
        }
    }

    // Export platform theme for login sessions when qt5ct/qt6ct is installed.
    if let Some(platform_theme) = detect_qt_platform_theme() {
        let envd_path = base_path.join("environment.d/90-sway-config-qt.conf");
        let envd_content = build_qt_envd_content(platform_theme);
        match write_file(&envd_path, &envd_content) {
            Ok(_) => result.files_written.push(envd_path.to_string_lossy().into()),
            Err(e) => result.errors.push(format!(
                "Failed to write environment.d QT platform theme config: {}",
                e
            )),
        }

        // Make Qt env vars effective for new app launches in the current session.
        import_qt_env_to_session(platform_theme);
    }

    // Set wallpaper via swaymsg
    if let Some(wp) = vars.get("WALLPAPER_PATH").filter(|p| !p.is_empty()) {
        let wp = wp.clone();
        let _ = std::process::Command::new("swaymsg")
            .args(["output", "*", "bg", &wp, "fill"])
            .status();
    }

    result.success = result.errors.is_empty();

    // Reload sway
    if result.success {
        result.sway_reloaded = reload_sway();
    }

    // Restart waybar
    if result.success {
        result.waybar_restarted = restart_waybar();
    }

    result
}

/// Resolve a path relative to $HOME/.config (not base_path, which is ~/.config in prod)
fn dirs_or_home(name: &str, base_path: &Path) -> PathBuf {
    // In tests base_path is a temp dir; in prod it is ~/.config
    base_path.join(name)
}

/// Build sway client color directives from theme variables.
fn build_sway_colors_conf(vars: &std::collections::HashMap<String, String>) -> String {
    let get = |k: &str| vars.get(k).map(|s| s.as_str()).unwrap_or("#888888");

    let base = get("COLOR_BASE");
    let text = get("COLOR_TEXT");
    let lavender = get("COLOR_LAVENDER");
    let overlay0 = get("COLOR_OVERLAY0");
    let subtext0 = get("COLOR_SUBTEXT0");
    let rosewater = get("COLOR_ROSEWATER");
    let peach = get("COLOR_PEACH");

    format!(
        "# Sway client colors — generated by sway-config\n\
         client.focused           {lavender}  {base}  {text}     {rosewater} {lavender}\n\
         client.focused_inactive  {overlay0}  {base}  {subtext0} {rosewater} {overlay0}\n\
         client.unfocused         {overlay0}  {base}  {subtext0} {rosewater} {overlay0}\n\
         client.urgent            {peach}     {base}  {peach}    {rosewater} {peach}\n\
         \n\
         # Gaps\n\
         gaps inner {gap_inner}\n\
         gaps outer {gap_outer}\n\
         default_border pixel {border_width}\n",
        lavender = lavender,
        base = base,
        text = text,
        rosewater = rosewater,
        overlay0 = overlay0,
        subtext0 = subtext0,
        peach = peach,
        gap_inner = vars.get("GAP_INNER").map(|s| s.as_str()).unwrap_or("5"),
        gap_outer = vars.get("GAP_OUTER").map(|s| s.as_str()).unwrap_or("5"),
        border_width = vars.get("BORDER_WIDTH").map(|s| s.as_str()).unwrap_or("2"),
    )
}

/// Build GTK settings.ini content from theme variables.
fn build_gtk_settings(vars: &std::collections::HashMap<String, String>) -> String {
    let theme = vars
        .get("GTK_THEME_NAME")
        .map(|s| s.as_str())
        .unwrap_or("Adwaita");
    let icons = vars
        .get("ICON_THEME")
        .map(|s| s.as_str())
        .unwrap_or("Adwaita");
    let font = vars
        .get("FONT_FAMILY")
        .map(|s| s.as_str())
        .unwrap_or("Sans");
    let size = vars.get("FONT_SIZE").map(|s| s.as_str()).unwrap_or("10");

    format!(
        "[Settings]\n\
         gtk-theme-name={theme}\n\
         gtk-icon-theme-name={icons}\n\
         gtk-font-name={font} {size}\n\
         gtk-cursor-theme-name=default\n\
         gtk-cursor-theme-size=24\n\
         gtk-xft-antialias=1\n\
         gtk-xft-hinting=1\n\
         gtk-xft-hintstyle=hintfull\n\
         gtk-xft-rgba=rgb\n",
        theme = theme,
        icons = icons,
        font = font,
        size = size,
    )
}

/// Build qt5ct/qt6ct config from theme variables.
fn build_qtct_settings(
    vars: &std::collections::HashMap<String, String>,
    color_scheme_path: &Path,
) -> String {
    let icons = vars
        .get("ICON_THEME")
        .map(|s| s.as_str())
        .unwrap_or("Adwaita");
    format!(
        "[Appearance]\n\
         color_scheme_path={color_scheme_path}\n\
         custom_palette=true\n\
         icon_theme={icons}\n\
         standard_dialogs=default\n\
         style=Fusion\n",
        color_scheme_path = color_scheme_path.to_string_lossy(),
        icons = icons,
    )
}

/// Build a qt5ct/qt6ct color scheme file.
fn build_qtct_color_scheme(vars: &std::collections::HashMap<String, String>) -> String {
    let active = build_qt_color_set(vars, "active");
    let disabled = build_qt_color_set(vars, "disabled");
    let inactive = build_qt_color_set(vars, "inactive");
    format!(
        "[ColorScheme]\nactive_colors={active}\ndisabled_colors={disabled}\ninactive_colors={inactive}\n",
        active = active.join(", "),
        disabled = disabled.join(", "),
        inactive = inactive.join(", "),
    )
}

fn build_qt_color_set(
    vars: &std::collections::HashMap<String, String>,
    state: &str,
) -> Vec<String> {
    let map = |k: &str, fallback: &str| {
        vars.get(k)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    };
    let (window, button, text, highlight, link, visited, placeholder) = match state {
        "disabled" => (
            map("COLOR_SURFACE1", "#44475a"),
            map("COLOR_SURFACE0", "#3a3f4b"),
            map("COLOR_OVERLAY0", "#8a8f98"),
            map("COLOR_SURFACE2", "#555b68"),
            map("COLOR_BLUE", "#7aa2f7"),
            map("COLOR_MAUVE", "#bb9af7"),
            map("COLOR_OVERLAY1", "#6c7086"),
        ),
        "inactive" => (
            map("COLOR_BASE", "#1e1e2e"),
            map("COLOR_SURFACE0", "#313244"),
            map("COLOR_SUBTEXT0", "#a6adc8"),
            map("COLOR_BLUE", "#89b4fa"),
            map("COLOR_BLUE", "#89b4fa"),
            map("COLOR_MAUVE", "#cba6f7"),
            map("COLOR_OVERLAY0", "#6c7086"),
        ),
        _ => (
            map("COLOR_BASE", "#1e1e2e"),
            map("COLOR_SURFACE0", "#313244"),
            map("COLOR_TEXT", "#cdd6f4"),
            map("COLOR_LAVENDER", "#b4befe"),
            map("COLOR_BLUE", "#89b4fa"),
            map("COLOR_MAUVE", "#cba6f7"),
            map("COLOR_OVERLAY0", "#6c7086"),
        ),
    };

    vec![
        qt_hex(&text),                              // WindowText
        qt_hex(&button),                            // Button
        qt_hex(&map("COLOR_SURFACE1", "#45475a")), // Light
        qt_hex(&map("COLOR_SURFACE2", "#585b70")), // Midlight
        qt_hex(&map("COLOR_MANTLE", "#181825")),   // Dark
        qt_hex(&map("COLOR_SURFACE2", "#585b70")), // Mid
        qt_hex(&text),                              // Text
        qt_hex(&map("COLOR_CRUST", "#11111b")),    // BrightText
        qt_hex(&text),                              // ButtonText
        qt_hex(&map("COLOR_BASE", "#1e1e2e")),     // Base
        qt_hex(&window),                            // Window
        qt_hex(&map("COLOR_MANTLE", "#181825")),   // Shadow
        qt_hex(&highlight),                         // Highlight
        qt_hex(&map("COLOR_BASE", "#1e1e2e")),     // HighlightedText
        qt_hex(&link),                              // Link
        qt_hex(&visited),                           // LinkVisited
        qt_hex(&map("COLOR_SURFACE0", "#313244")), // AlternateBase
        qt_hex(&map("COLOR_SURFACE0", "#313244")), // ToolTipBase
        qt_hex(&text),                              // ToolTipText
        qt_hex(&placeholder),                       // PlaceholderText
        qt_hex_alpha(&map("COLOR_LAVENDER", "#b4befe"), 0x80), // Accent
    ]
}

fn qt_hex(color: &str) -> String {
    let c = color.trim();
    if c.starts_with('#') && c.len() == 7 {
        format!("#ff{}", &c[1..])
    } else if c.starts_with('#') && c.len() == 9 {
        c.to_string()
    } else {
        "#ff000000".to_string()
    }
}

fn qt_hex_alpha(color: &str, alpha: u8) -> String {
    let c = color.trim();
    if c.starts_with('#') && c.len() == 7 {
        format!("#{alpha:02x}{}", &c[1..], alpha = alpha)
    } else if c.starts_with('#') && c.len() == 9 {
        format!("#{:02x}{}", alpha, &c[3..])
    } else {
        format!("#{:02x}000000", alpha)
    }
}

fn command_exists(bin: &str) -> bool {
    std::process::Command::new("which")
        .arg(bin)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn detect_qt_platform_theme() -> Option<&'static str> {
    if command_exists("qt5ct") {
        Some("qt5ct")
    } else if command_exists("qt6ct") {
        Some("qt6ct")
    } else {
        None
    }
}

fn build_qt_envd_content(platform_theme: &str) -> String {
    format!(
        "QT_QPA_PLATFORMTHEME={platform_theme}\nQT_STYLE_OVERRIDE=Fusion\n",
        platform_theme = platform_theme
    )
}

fn import_qt_env_to_session(platform_theme: &str) {
    use std::process::Command;
    let qt_env = format!("QT_QPA_PLATFORMTHEME={}", platform_theme);
    let style_env = "QT_STYLE_OVERRIDE=Fusion";

    let _ = Command::new("dbus-update-activation-environment")
        .args(["--systemd", &qt_env, style_env])
        .status();

    let _ = Command::new("systemctl")
        .env("QT_QPA_PLATFORMTHEME", platform_theme)
        .env("QT_STYLE_OVERRIDE", "Fusion")
        .args([
            "--user",
            "import-environment",
            "QT_QPA_PLATFORMTHEME",
            "QT_STYLE_OVERRIDE",
        ])
        .status();
}

/// Embedded waybar style.css template (tokens in ${VAR} form).
const WAYBAR_STYLE_TEMPLATE: &str = include_str!("../assets/waybar_style.css.tmpl");

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

    #[test]
    fn test_build_qtct_settings_uses_theme_vars() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("ICON_THEME".to_string(), "Papirus".to_string());

        let conf = build_qtct_settings(&vars, Path::new("/tmp/sway-config.conf"));
        assert!(conf.contains("icon_theme=Papirus"));
        assert!(conf.contains("custom_palette=true"));
    }

    #[test]
    fn test_build_qt_envd_content_includes_theme_and_style() {
        let envd = build_qt_envd_content("qt5ct");
        assert!(envd.contains("QT_QPA_PLATFORMTHEME=qt5ct"));
        assert!(envd.contains("QT_STYLE_OVERRIDE=Fusion"));
    }

    #[test]
    fn test_build_qtct_color_scheme_has_three_palettes() {
        let vars = std::collections::HashMap::new();
        let conf = build_qtct_color_scheme(&vars);
        assert!(conf.contains("[ColorScheme]"));
        assert!(conf.contains("active_colors="));
        assert!(conf.contains("disabled_colors="));
        assert!(conf.contains("inactive_colors="));
    }
}
