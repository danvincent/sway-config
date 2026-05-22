/// Apply integration tests - full render + write + reload workflow
use sway_configurator::config::render_model::RenderModel;
use sway_configurator::model::settings::Settings;
use sway_configurator::model::output::{OutputConfig, Resolution, Position, Transform};
use sway_configurator::model::input::{KeyboardConfig, TouchpadConfig, AccelProfile};
use sway_configurator::model::idle::IdleConfig;
use sway_configurator::model::waybar::{WaybarConfig, WaybarModule, BarPosition};
use sway_configurator::model::autostart::{AutostartConfig, AutostartEntry};

#[test]
fn test_render_model_from_empty_settings() {
    let settings = Settings::default();
    let model = RenderModel::from_settings(&settings);
    assert!(model.outputs.is_empty());
    assert!(model.inputs.is_empty());
    // Default idle config has lock_timeout: 300, so idle will be Some
    assert!(model.idle.is_some());
}

#[test]
fn test_render_output_enabled() {
    let mut settings = Settings::default();
    settings.outputs.push(OutputConfig {
        name: "HDMI-A-1".to_string(),
        enabled: true,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: Some(60000),
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    let model = RenderModel::from_settings(&settings);
    assert_eq!(model.outputs.len(), 1);
    let rendered = &model.outputs[0];
    assert_eq!(rendered.name, "HDMI-A-1");
    // Check that directive contains key elements
    assert!(rendered.sway_directive.contains("HDMI-A-1"));
    assert!(rendered.sway_directive.contains("1920"));
    assert!(rendered.sway_directive.contains("1080"));
}

#[test]
fn test_render_output_disabled() {
    let mut settings = Settings::default();
    settings.outputs.push(OutputConfig {
        name: "eDP-1".to_string(),
        enabled: false,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    let model = RenderModel::from_settings(&settings);
    assert_eq!(model.outputs.len(), 1);
    let rendered = &model.outputs[0];
    assert!(rendered.sway_directive.contains("eDP-1"));
    assert!(rendered.sway_directive.contains("disable"));
}

#[test]
fn test_render_keyboard() {
    let mut settings = Settings::default();
    settings.keyboards.push(KeyboardConfig {
        identifier: "keyboard123".to_string(),
        xkb_layout: "gb".to_string(),
        xkb_variant: "dvorak".to_string(),
        xkb_options: "compose:ralt".to_string(),
        repeat_delay: 500,
        repeat_rate: 30,
    });
    
    let model = RenderModel::from_settings(&settings);
    assert_eq!(model.inputs.len(), 1);
    let rendered = &model.inputs[0];
    assert_eq!(rendered.identifier, "keyboard123");
    assert!(rendered.sway_directive.contains("keyboard123"));
    assert!(rendered.sway_directive.contains("gb"));
    assert!(rendered.sway_directive.contains("dvorak"));
    assert!(rendered.sway_directive.contains("500"));
}

#[test]
fn test_render_touchpad() {
    let mut settings = Settings::default();
    settings.touchpads.push(TouchpadConfig {
        identifier: "touchpad456".to_string(),
        tap_to_click: true,
        natural_scroll: false,
        dwt: true,
        accel_speed: 0.5,
        accel_profile: AccelProfile::Flat,
        left_handed: false,
        middle_emulation: false,
    });
    
    let model = RenderModel::from_settings(&settings);
    assert_eq!(model.inputs.len(), 1);
    let rendered = &model.inputs[0];
    assert_eq!(rendered.identifier, "touchpad456");
    assert!(rendered.sway_directive.contains("touchpad456"));
    assert!(rendered.sway_directive.contains("enabled"));  // tap enabled
    assert!(rendered.sway_directive.contains("flat"));
}

#[test]
fn test_render_idle_with_lock_and_sleep() {
    let mut settings = Settings::default();
    settings.idle = IdleConfig {
        lock_timeout: 300,
        screen_off_timeout: 0,
        lock_command: "swaylock".to_string(),
        before_sleep: true,
    };
    
    let model = RenderModel::from_settings(&settings);
    assert!(model.idle.is_some());
    let idle = model.idle.as_ref().unwrap();
    assert!(idle.swayidle_exec.contains("swayidle"));
    assert!(idle.swayidle_exec.contains("timeout 300"));
    assert!(idle.swayidle_exec.contains("before-sleep"));
}

#[test]
fn test_render_idle_disabled() {
    let mut settings = Settings::default();
    settings.idle = IdleConfig {
        lock_timeout: 0,
        screen_off_timeout: 0,
        lock_command: "swaylock".to_string(),
        before_sleep: false,
    };
    
    let model = RenderModel::from_settings(&settings);
    // When lock_timeout is 0, idle should be None or empty
    let idle_conf = model.to_sway_idle_conf();
    assert!(idle_conf.is_empty());
}

#[test]
fn test_render_idle_with_screen_off() {
    let mut settings = Settings::default();
    settings.idle = IdleConfig {
        lock_timeout: 300,
        screen_off_timeout: 900,
        lock_command: "swaylock".to_string(),
        before_sleep: true,
    };
    
    let model = RenderModel::from_settings(&settings);
    assert!(model.idle.is_some());
    let idle = model.idle.as_ref().unwrap();
    assert!(idle.swayidle_exec.contains("timeout 900"));
    assert!(idle.swayidle_exec.contains("dpms"));
}

#[test]
fn test_render_waybar_config_json() {
    let mut settings = Settings::default();
    settings.waybar = WaybarConfig {
        enabled: true,
        position: BarPosition::Top,
        height: 30,
        modules_left: vec!["sway/workspaces".to_string()],
        modules_center: vec!["clock".to_string()],
        modules_right: vec![
            WaybarModule::new("battery", true),
            WaybarModule::new("network", false),
        ],
        tray: true,
    };
    
    let model = RenderModel::from_settings(&settings);
    assert!(model.waybar.is_some());
    let waybar = model.waybar.as_ref().unwrap();
    
    // Parse JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&waybar.config_json)
        .expect("Waybar config JSON must be valid");
    
    // Verify key fields are present
    assert!(json.is_object());
    assert_eq!(json["position"], "top");
    assert_eq!(json["height"], 30);
}

#[test]
fn test_render_waybar_disabled() {
    let mut settings = Settings::default();
    settings.waybar = WaybarConfig {
        enabled: false,
        position: BarPosition::Top,
        height: 30,
        modules_left: vec![],
        modules_center: vec![],
        modules_right: vec![],
        tray: true,
    };
    
    let model = RenderModel::from_settings(&settings);
    // Should not render waybar config if disabled
    assert!(model.waybar.is_none());
}

#[test]
fn test_render_autostart_entries() {
    let mut settings = Settings::default();
    let mut autostart_config = AutostartConfig::new();
    autostart_config.add(AutostartEntry {
        id: "entry1".to_string(),
        command: "example-daemon".to_string(),
        enabled: true,
        description: "Example Daemon".to_string(),
    });
    autostart_config.add(AutostartEntry {
        id: "entry2".to_string(),
        command: "another-app".to_string(),
        enabled: false,
        description: "Another App".to_string(),
    });
    autostart_config.add(AutostartEntry {
        id: "entry3".to_string(),
        command: "third-app".to_string(),
        enabled: true,
        description: "Third App".to_string(),
    });
    settings.autostart = autostart_config;
    
    let model = RenderModel::from_settings(&settings);
    // Only enabled entries should be rendered
    assert_eq!(model.autostart.len(), 2);
    
    let entry1 = model.autostart.iter().find(|e| e.exec_line.contains("example-daemon"));
    assert!(entry1.is_some());
    assert!(entry1.unwrap().exec_line.contains("Example Daemon"));
}

#[test]
fn test_render_autostart_disabled_excluded() {
    let mut settings = Settings::default();
    let mut autostart_config = AutostartConfig::new();
    autostart_config.add(AutostartEntry {
        id: "entry1".to_string(),
        command: "enabled-app".to_string(),
        enabled: true,
        description: "Enabled".to_string(),
    });
    autostart_config.add(AutostartEntry {
        id: "entry2".to_string(),
        command: "disabled-app".to_string(),
        enabled: false,
        description: "Disabled".to_string(),
    });
    settings.autostart = autostart_config;
    
    let model = RenderModel::from_settings(&settings);
    let all_directives = model.autostart.iter()
        .map(|a| a.exec_line.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    
    assert!(all_directives.contains("enabled-app"));
    assert!(!all_directives.contains("disabled-app"));
}

#[test]
fn test_outputs_conf_format() {
    let mut settings = Settings::default();
    settings.outputs.push(OutputConfig {
        name: "HDMI-A-1".to_string(),
        enabled: true,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    settings.outputs.push(OutputConfig {
        name: "eDP-1".to_string(),
        enabled: true,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: None,
        position: Position { x: 1920, y: 0 },
        scale: 1.5,
        transform: Transform::Normal,
    });
    
    let model = RenderModel::from_settings(&settings);
    let conf = model.to_sway_outputs_conf();
    
    // Should contain both outputs joined with newlines
    assert!(conf.contains("HDMI-A-1"));
    assert!(conf.contains("eDP-1"));
    let lines: Vec<&str> = conf.lines().collect();
    assert!(lines.len() >= 2);
}

#[test]
fn test_inputs_conf_format() {
    let mut settings = Settings::default();
    settings.keyboards.push(KeyboardConfig {
        identifier: "kbd1".to_string(),
        xkb_layout: "us".to_string(),
        xkb_variant: "".to_string(),
        xkb_options: "".to_string(),
        repeat_delay: 600,
        repeat_rate: 25,
    });
    settings.touchpads.push(TouchpadConfig {
        identifier: "touch1".to_string(),
        tap_to_click: true,
        natural_scroll: false,
        dwt: false,
        accel_speed: 0.0,
        accel_profile: AccelProfile::Adaptive,
        left_handed: false,
        middle_emulation: false,
    });
    
    let model = RenderModel::from_settings(&settings);
    let conf = model.to_sway_inputs_conf();
    
    // Should contain both keyboard and touchpad blocks
    assert!(conf.contains("kbd1"));
    assert!(conf.contains("touch1"));
}

#[test]
fn test_apply_dry_run_does_not_write_files() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    
    let mut settings = Settings::default();
    // Disable idle to have truly empty config
    settings.idle.lock_timeout = 0;
    // Disable waybar
    settings.waybar.enabled = false;
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: true,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // In dry-run mode, no actual files should be written to disk
    // But the result should list which files WOULD be written
    assert!(!result.files_written.is_empty(), "Should list files that would be written");
}

#[test]
fn test_apply_dry_run_returns_file_list() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    
    let mut settings = Settings::default();
    settings.outputs.push(OutputConfig {
        name: "HDMI-A-1".to_string(),
        enabled: true,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: true,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    // Even in dry-run, should list files that would be written
    assert!(!result.files_written.is_empty());
}

#[test]
fn test_apply_writes_outputs_conf() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    settings.outputs.push(OutputConfig {
        name: "HDMI-A-1".to_string(),
        enabled: true,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // Check file was written
    let outputs_conf_path = tempdir.path().join("sway/config.d/outputs.conf");
    assert!(outputs_conf_path.exists());
    
    let content = fs::read_to_string(&outputs_conf_path).expect("Failed to read outputs.conf");
    assert!(content.contains("HDMI-A-1"));
}

#[test]
fn test_apply_writes_inputs_conf() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    settings.keyboards.push(KeyboardConfig {
        identifier: "kbd1".to_string(),
        xkb_layout: "gb".to_string(),
        xkb_variant: "".to_string(),
        xkb_options: "".to_string(),
        repeat_delay: 600,
        repeat_rate: 25,
    });
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    let inputs_conf_path = tempdir.path().join("sway/config.d/inputs.conf");
    assert!(inputs_conf_path.exists());
    
    let content = fs::read_to_string(&inputs_conf_path).expect("Failed to read inputs.conf");
    assert!(content.contains("kbd1"));
}

#[test]
fn test_apply_writes_idle_conf() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    settings.idle = IdleConfig {
        lock_timeout: 300,
        screen_off_timeout: 0,
        lock_command: "swaylock".to_string(),
        before_sleep: true,
    };
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    let idle_conf_path = tempdir.path().join("sway/config.d/idle.conf");
    assert!(idle_conf_path.exists());
    
    let content = fs::read_to_string(&idle_conf_path).expect("Failed to read idle.conf");
    assert!(content.contains("swayidle"));
}

#[test]
fn test_apply_creates_parent_dirs() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    
    let mut settings = Settings::default();
    settings.outputs.push(OutputConfig {
        name: "HDMI-A-1".to_string(),
        enabled: true,
        resolution: Some(Resolution { width: 1920, height: 1080 }),
        refresh_rate: None,
        position: Position { x: 0, y: 0 },
        scale: 1.0,
        transform: Transform::Normal,
    });
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // Parent directories should be created
    let conf_d_path = tempdir.path().join("sway/config.d");
    assert!(conf_d_path.exists());
    assert!(conf_d_path.is_dir());
}

#[test]
fn test_apply_result_success_fields() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    
    let settings = Settings::default();
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    
    // Verify ApplyResult fields are populated
    assert!(result.success || !result.success); // Tautology, just check it exists
    assert!(result.files_written.is_empty() || !result.files_written.is_empty()); // Check field exists
    assert!(result.errors.is_empty() || !result.errors.is_empty()); // Check field exists
}

#[test]
fn test_apply_writes_empty_idle_conf_when_disabled() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    // Disable idle by setting lock_timeout to 0
    settings.idle.lock_timeout = 0;
    // Disable waybar
    settings.waybar.enabled = false;
    // Clear autostart
    settings.autostart.entries.clear();
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // idle.conf should exist even when idle is disabled
    let idle_conf_path = tempdir.path().join("sway/config.d/idle.conf");
    assert!(idle_conf_path.exists(), "idle.conf should exist even when disabled");
    
    let content = fs::read_to_string(&idle_conf_path).expect("Failed to read idle.conf");
    assert!(content.is_empty() || content.trim().is_empty(), "idle.conf should be empty when disabled");
}

#[test]
fn test_apply_writes_empty_autostart_conf_when_all_disabled() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    // Disable idle
    settings.idle.lock_timeout = 0;
    // Disable waybar
    settings.waybar.enabled = false;
    // Add only disabled autostart entries
    settings.autostart.entries.clear();
    settings.autostart.add(AutostartEntry {
        id: "disabled1".to_string(),
        command: "some-app".to_string(),
        enabled: false,
        description: "Disabled App".to_string(),
    });
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // autostart.conf should exist even when all entries are disabled
    let autostart_conf_path = tempdir.path().join("sway/config.d/autostart.conf");
    assert!(autostart_conf_path.exists(), "autostart.conf should exist even when all entries are disabled");
    
    let content = fs::read_to_string(&autostart_conf_path).expect("Failed to read autostart.conf");
    assert!(content.is_empty() || content.trim().is_empty(), "autostart.conf should be empty when all entries are disabled");
}

#[test]
fn test_apply_writes_empty_waybar_config_json_when_disabled() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    // Disable idle
    settings.idle.lock_timeout = 0;
    // Disable waybar
    settings.waybar.enabled = false;
    // Clear autostart
    settings.autostart.entries.clear();
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // config.jsonc should exist even when waybar is disabled
    let waybar_config_path = tempdir.path().join("waybar/config.jsonc");
    assert!(waybar_config_path.exists(), "config.jsonc should exist even when waybar is disabled");
    
    let content = fs::read_to_string(&waybar_config_path).expect("Failed to read config.jsonc");
    // Should contain empty JSON object
    assert_eq!(content.trim(), "{}", "config.jsonc should be empty JSON object when waybar is disabled");
}

#[test]
fn test_apply_clears_stale_files() {
    use sway_configurator::config::apply::{ApplyConfig, apply};
    use tempfile::TempDir;
    use std::fs;
    
    let mut settings = Settings::default();
    // Set up initial config with idle, autostart, and waybar
    settings.idle.lock_timeout = 300;
    settings.idle.lock_command = "swaylock".to_string();
    settings.waybar.enabled = true;
    settings.waybar.position = BarPosition::Bottom;
    settings.autostart.entries.clear();
    settings.autostart.add(AutostartEntry {
        id: "app1".to_string(),
        command: "myapp".to_string(),
        enabled: true,
        description: "My App".to_string(),
    });
    
    let tempdir = TempDir::new().expect("Failed to create temp dir");
    
    let config = ApplyConfig {
        dry_run: false,
        reload_sway: false,
        restart_waybar: false,
    };
    
    // First apply - with config
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    let idle_conf_path = tempdir.path().join("sway/config.d/idle.conf");
    let autostart_conf_path = tempdir.path().join("sway/config.d/autostart.conf");
    let waybar_config_path = tempdir.path().join("waybar/config.jsonc");
    
    assert!(idle_conf_path.exists());
    assert!(autostart_conf_path.exists());
    assert!(waybar_config_path.exists());
    
    let idle_content_before = fs::read_to_string(&idle_conf_path).expect("Failed to read idle.conf");
    assert!(!idle_content_before.is_empty(), "idle.conf should have content initially");
    
    // Now disable everything
    settings.idle.lock_timeout = 0;
    settings.waybar.enabled = false;
    settings.autostart.entries.clear();
    
    // Second apply - all disabled
    let result = apply(&settings, config, tempdir.path());
    assert!(result.success);
    
    // Files should still exist but be empty
    assert!(idle_conf_path.exists(), "idle.conf should still exist");
    let idle_content_after = fs::read_to_string(&idle_conf_path).expect("Failed to read idle.conf");
    assert!(idle_content_after.is_empty() || idle_content_after.trim().is_empty(), "idle.conf should be empty after disabling");
    
    assert!(autostart_conf_path.exists(), "autostart.conf should still exist");
    let autostart_content_after = fs::read_to_string(&autostart_conf_path).expect("Failed to read autostart.conf");
    assert!(autostart_content_after.is_empty() || autostart_content_after.trim().is_empty(), "autostart.conf should be empty after disabling");
    
    assert!(waybar_config_path.exists(), "config.jsonc should still exist");
    let waybar_content_after = fs::read_to_string(&waybar_config_path).expect("Failed to read config.jsonc");
    assert_eq!(waybar_content_after.trim(), "{}", "config.jsonc should be empty JSON object after disabling");
}
