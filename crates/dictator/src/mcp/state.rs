//! Server state and configuration management.

use mcp_host::managers::progress::ProgressTracker;
use mcp_host::protocol::types::ClientInfo;
use notify::RecommendedWatcher;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

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
#[derive(Debug, Default, Clone, Deserialize)]
pub struct DictateConfig {
    #[serde(default)]
    pub decree: HashMap<String, DecreeSettings>,
}

/// Settings for a specific decree (language)
#[derive(Debug, Default, Clone, Deserialize)]
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
#[allow(clippy::struct_excessive_bools)]
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
    // Config file change detection
    pub config_dirty: bool,
    // Progress tracking for long-running operations
    pub progress_tracker: Arc<ProgressTracker>,
    // Violations seen by the most recent lint (tool call or watch check)
    pub last_violation_total: usize,
    // Set when the mood tier changed; drained by the watcher loop to
    // emit resources/updated for dictator://mood
    pub mood_dirty: bool,
}

/// The Dictator's disposition, derived from the last lint result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// Zero violations: the timeline is compliant
    Magnanimous,
    /// A few violations: patience wears thin
    Displeased,
    /// Ten or more violations: open rebellion
    Wrathful,
}

impl Mood {
    /// Mood tier for a violation count
    pub fn from_violations(total: usize) -> Self {
        match total {
            0 => Self::Magnanimous,
            1..=9 => Self::Displeased,
            _ => Self::Wrathful,
        }
    }

    /// Stable machine-readable label
    pub fn label(self) -> &'static str {
        match self {
            Self::Magnanimous => "magnanimous",
            Self::Displeased => "displeased",
            Self::Wrathful => "wrathful",
        }
    }

    /// Official proclamation for the current mood
    pub fn proclamation(self) -> &'static str {
        match self {
            Self::Magnanimous => {
                "The people write compliant code. The Dictator smiles upon the timeline."
            }
            Self::Displeased => "Violations detected. The Dictator's patience wears thin.",
            Self::Wrathful => "The codebase is in open rebellion. Re-education is imminent.",
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        // Create a dummy notification channel for default initialization
        // This is only used in tests; actual server uses ::new()
        let (tx, _rx) = mcp_host::NotificationSender::bounded(1);
        Self::new(tx)
    }
}

impl ServerState {
    /// Create new `ServerState` with notification channel
    pub fn new(notif_tx: mcp_host::NotificationSender) -> Self {
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
            config_dirty: false,
            progress_tracker: Arc::new(ProgressTracker::new(notif_tx)),
            last_violation_total: 0,
            mood_dirty: false,
        }
    }

    /// Record a lint result, flagging `mood_dirty` when the mood tier changes
    pub fn record_violations(&mut self, total: usize) {
        let previous = Mood::from_violations(self.last_violation_total);
        self.last_violation_total = total;
        if Mood::from_violations(total) != previous {
            self.mood_dirty = true;
        }
    }

    /// Current mood derived from the last lint result
    pub fn mood(&self) -> Mood {
        Mood::from_violations(self.last_violation_total)
    }

    /// Reload configuration from disk
    pub fn reload_config(&mut self) {
        let old_config = self.config.take();
        self.config = load_config();

        if let Some(ref cfg) = self.config {
            log_to_file(&format!("Reloaded config with {} decrees", cfg.decree.len()));
        } else {
            log_to_file("Config removed or invalid");
        }

        // Check if linter availability changed (affects tool list)
        let old_has_linter =
            old_config.as_ref().is_some_and(|c| c.decree.values().any(|d| d.linter.is_some()));
        let new_has_linter =
            self.config.as_ref().is_some_and(|c| c.decree.values().any(|d| d.linter.is_some()));

        if old_has_linter != new_has_linter {
            log_to_file(&format!(
                "Linter availability changed: {old_has_linter} -> {new_has_linter}"
            ));
        }
    }

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
