/// Autostart configuration - programs to launch on startup
use serde::{Deserialize, Serialize};

/// A single autostart entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutostartEntry {
    /// Unique identifier for this entry (UUID or slug)
    pub id: String,
    /// Command to execute
    pub command: String,
    /// Whether this entry is enabled
    pub enabled: bool,
    /// User-facing description
    pub description: String,
}

/// Collection of autostart entries
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutostartConfig {
    /// List of autostart entries
    pub entries: Vec<AutostartEntry>,
}

impl AutostartConfig {
    /// Create a new empty autostart config
    pub fn new() -> Self {
        AutostartConfig {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the autostart config
    pub fn add(&mut self, entry: AutostartEntry) {
        self.entries.push(entry);
    }

    /// Remove an entry by id
    pub fn remove(&mut self, id: &str) {
        self.entries.retain(|entry| entry.id != id);
    }

    /// Find an entry by id
    pub fn find(&self, id: &str) -> Option<&AutostartEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }
}
