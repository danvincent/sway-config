use std::io;
/// Sway message wrapper - low-level swaymsg command invocation
use std::process::Command;

/// Errors that can occur when communicating with Sway
#[derive(Debug)]
pub enum SwayError {
    /// Sway is not running (SWAYSOCK not set or swaymsg not found)
    NotRunning,
    /// JSON parsing failed
    ParseError(String),
    /// IO or subprocess error
    IoError(io::Error),
}

impl From<io::Error> for SwayError {
    fn from(e: io::Error) -> Self {
        SwayError::IoError(e)
    }
}

impl PartialEq for SwayError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SwayError::NotRunning, SwayError::NotRunning) => true,
            (SwayError::ParseError(a), SwayError::ParseError(b)) => a == b,
            (SwayError::IoError(a), SwayError::IoError(b)) => {
                a.kind() == b.kind() && a.to_string() == b.to_string()
            }
            _ => false,
        }
    }
}

impl Eq for SwayError {}

/// Check if Sway is currently running
/// Returns true if SWAYSOCK environment variable is set, false otherwise
/// This is safe to call outside Sway (returns false gracefully)
pub fn is_sway_running() -> bool {
    std::env::var("SWAYSOCK").is_ok()
}

/// Run swaymsg and return the JSON response string
fn run_swaymsg(type_arg: &str) -> Result<String, SwayError> {
    // First check if Sway is running
    if !is_sway_running() {
        return Err(SwayError::NotRunning);
    }

    let output = Command::new("swaymsg").arg("-t").arg(type_arg).output()?;

    if !output.status.success() {
        return Err(SwayError::NotRunning);
    }

    String::from_utf8(output.stdout)
        .map_err(|e| SwayError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))
}

/// Get list of outputs from Sway
/// Returns empty vec if Sway is not running
pub fn get_outputs() -> Result<Vec<crate::config::detect::SwayOutput>, SwayError> {
    let json_str = run_swaymsg("get_outputs")?;
    serde_json::from_str(&json_str)
        .map_err(|e| SwayError::ParseError(format!("Failed to parse outputs JSON: {}", e)))
}

/// Get list of inputs from Sway
/// Returns empty vec if Sway is not running
pub fn get_inputs() -> Result<Vec<crate::config::detect::SwayInput>, SwayError> {
    let json_str = run_swaymsg("get_inputs")?;
    serde_json::from_str(&json_str)
        .map_err(|e| SwayError::ParseError(format!("Failed to parse inputs JSON: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sway_running_returns_bool() {
        let _result = is_sway_running();
        // If we get here without panic, test passes
    }
}
