/// Feature detection - determines available hardware and software capabilities
use std::fs;
use std::path::Path;

/// Returns true if `swayidle` binary exists on PATH
pub fn has_swayidle() -> bool {
    which::which("swayidle").is_ok()
}

/// Returns true if `waybar` binary exists on PATH
pub fn has_waybar() -> bool {
    which::which("waybar").is_ok()
}

/// Returns true if a screen locker binary exists on PATH
/// Checks for: swaylock, swaylock-effects, waylock
pub fn has_screen_locker() -> bool {
    which::which("swaylock").is_ok()
        || which::which("swaylock-effects").is_ok()
        || which::which("waylock").is_ok()
}

/// Returns true if a battery/power supply exists under /sys/class/power_supply/
pub fn has_battery() -> bool {
    match fs::read_dir("/sys/class/power_supply/") {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("BAT") {
                        return true;
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Returns true if audio system is available
/// Checks for: pactl, pipewire-pulse, or alsa
pub fn has_audio() -> bool {
    which::which("pactl").is_ok()
        || which::which("pipewire-pulse").is_ok()
        || which::which("aplay").is_ok()
        || Path::new("/proc/asound").exists()
}

/// Returns true if a network manager is running/installed
/// Checks for: NetworkManager, iwd, wpa_supplicant
pub fn has_network_manager() -> bool {
    which::which("nmcli").is_ok()
        || which::which("iwctl").is_ok()
        || which::which("wpa_cli").is_ok()
        || which::which("wpa_supplicant").is_ok()
}

/// Returns true if a notification daemon is running/installed
/// Checks for: dunst, mako, swaync
pub fn has_notification_daemon() -> bool {
    which::which("dunst").is_ok() || which::which("mako").is_ok() || which::which("swaync").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_battery_returns_bool() {
        let _ = has_battery(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_has_audio_returns_bool() {
        let _ = has_audio(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_has_network_manager_returns_bool() {
        let _ = has_network_manager(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_has_notification_daemon_returns_bool() {
        let _ = has_notification_daemon(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_has_swayidle_returns_bool() {
        let _ = has_swayidle(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_has_waybar_returns_bool() {
        let _ = has_waybar(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_has_screen_locker_returns_bool() {
        let _ = has_screen_locker(); // Just ensure it doesn't panic
    }
}
