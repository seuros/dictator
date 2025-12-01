//! Server state and configuration management.

use notify::RecommendedWatcher;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::protocol::ClientInfo;
use super::utils::log_to_file;

pub const CONFIG_FILE: &str = ".dictate.toml";

/// Interval for checking if watched paths have changes (in seconds).
pub const WATCHER_CHECK_INTERVAL_SECS: u64 = 10;

/// Timeout for stalint violation check (in seconds).
pub const STALINT_CHECK_TIMEOUT_SECS: u64 = 60;

/// Default limit for stalint violations returned per request.
pub const DEFAULT_STALINT_LIMIT: usize = 10;

/// Linter configuration for a specific language
/// Only the command is specified - Dictator controls the args
#[derive(Debug, Clone, Deserialize)]
pub struct LinterConfig {
    pub command: String,
}

/// Decree configuration from .dictate.toml
#[derive(Debug, Default, Deserialize)]
pub struct DictateConfig {
    #[serde(default)]
    pub decree: HashMap<String, DecreeSettings>,
}

/// Settings for a specific decree (language)
#[derive(Debug, Default, Deserialize)]
pub struct DecreeSettings {
    #[serde(default)]
    pub linter: Option<LinterConfig>,
}

/// Load and parse .dictate.toml from cwd
pub fn load_config() -> Option<DictateConfig> {
    let cwd = std::env::current_dir().ok()?;
    let config_path = cwd.join(CONFIG_FILE);

    if !config_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&config_path).ok()?;
    toml::from_str(&content).ok()
}

/// Server state shared between handlers
pub struct ServerState {
    // Watcher state
    pub paths: HashSet<String>,
    pub dirty: bool,
    pub last_check: Instant,
    pub is_watching: bool,
    #[allow(dead_code)]
    pub watcher: Option<RecommendedWatcher>,
    // Client info
    pub client: ClientInfo,
    // Sandbox detection
    pub can_write: bool,
    // Pagination state for stalint
    pub stalint_paths: Vec<String>,
    // Linter configuration
    pub config: Option<DictateConfig>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            paths: HashSet::new(),
            dirty: false,
            last_check: Instant::now(),
            is_watching: false,
            watcher: None,
            client: ClientInfo::default(),
            // Default to writable; Codex and other clients can refine this
            // via sandbox state notifications once the MCP handshake is
            // complete. This keeps initialization lightweight.
            can_write: true,
            stalint_paths: Vec::new(),
            config: None,
        }
    }
}

impl ServerState {
    /// Lazily load .dictate.toml after the MCP handshake has completed.
    pub fn ensure_config_loaded(&mut self) {
        if self.config.is_some() {
            return;
        }

        let config = load_config();
        if let Some(ref cfg) = config {
            log_to_file(&format!("Loaded config with {} decrees", cfg.decree.len()));
        }
        self.config = config;
    }
}
