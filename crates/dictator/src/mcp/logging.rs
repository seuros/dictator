//! MCP structured logging with client-controlled verbosity and rate limiting.
//!
//! Implements RFC 5424 syslog severity levels (8 levels) with:
//! - Client-controlled minimum severity via `logging/setLevel`
//! - Token bucket rate limiting (100 msgs/10s window)
//! - Structured JSON payloads for details
//! - Thread-safe configuration updates

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// RFC 5424 Syslog severity levels (8 levels, 0=lowest, 7=highest verbosity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// System is unusable (least verbose, only show critical issues)
    Emergency = 0,
    /// Action must be taken immediately
    Alert = 1,
    /// Critical conditions
    Critical = 2,
    /// Error conditions
    Error = 3,
    /// Warning conditions
    Warning = 4,
    /// Normal but significant events (default level)
    Notice = 5,
    /// Informational messages
    Info = 6,
    /// Detailed debugging information (most verbose)
    Debug = 7,
}

impl Severity {
    /// Parse severity level from string (case-insensitive)
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "emergency" => Some(Severity::Emergency),
            "alert" => Some(Severity::Alert),
            "critical" => Some(Severity::Critical),
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "notice" => Some(Severity::Notice),
            "info" => Some(Severity::Info),
            "debug" => Some(Severity::Debug),
            _ => None,
        }
    }

    /// Get string representation of severity level
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Emergency => "emergency",
            Severity::Alert => "alert",
            Severity::Critical => "critical",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Notice => "notice",
            Severity::Info => "info",
            Severity::Debug => "debug",
        }
    }
}

/// Token bucket rate limiter: 100 tokens per 10 second window
#[derive(Debug)]
pub struct RateLimiter {
    #[allow(dead_code)]
    capacity: usize,
    #[allow(dead_code)]
    window_secs: u64,
    #[allow(dead_code)]
    tokens: Arc<AtomicUsize>,
    #[allow(dead_code)]
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create rate limiter with 100 tokens per 10 second window
    pub fn new() -> Self {
        Self {
            capacity: 100,
            window_secs: 10,
            tokens: Arc::new(AtomicUsize::new(100)),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Try to consume a token; returns true if allowed, false if rate-limited
    #[allow(dead_code)]
    pub fn try_log(&self) -> bool {
        // Refill tokens if window has passed
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed >= Duration::from_secs(self.window_secs) {
            self.tokens.store(self.capacity, Ordering::SeqCst);
            *last = now;
        }

        // Try to consume a token
        let mut current = self.tokens.load(Ordering::SeqCst);
        while current > 0 {
            let new = current - 1;
            match self
                .tokens
                .compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }

        false
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging configuration (shared, client-controlled)
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Minimum severity level to send (client-controlled via logging/setLevel)
    #[allow(dead_code)]
    pub min_level: Severity,
    /// Rate limiter to prevent channel overflow
    #[allow(dead_code)]
    pub rate_limiter: Arc<RateLimiter>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: Severity::Warning,
            rate_limiter: Arc::new(RateLimiter::new()),
        }
    }
}

/// Logger for sending structured log messages via MCP notifications
pub struct Logger {
    config: Arc<Mutex<LoggerConfig>>,
    notif_tx: tokio::sync::mpsc::Sender<String>,
}

impl Logger {
    /// Create new logger
    pub fn new(
        config: Arc<Mutex<LoggerConfig>>,
        notif_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Self {
        Self { config, notif_tx }
    }

    /// Log a message with severity and optional structured details
    /// Returns true if message was sent, false if rate-limited or severity filtered
    pub fn log(&self, severity: Severity, message: &str, details: Option<Value>) -> bool {
        let config = self.config.lock().unwrap();

        // Check if severity meets minimum threshold
        if severity > config.min_level {
            return false;
        }

        // Check rate limit
        if !config.rate_limiter.try_log() {
            return false;
        }

        // Build log notification
        let mut data = if let Some(d) = details {
            d.clone()
        } else {
            serde_json::json!({})
        };

        // Add message if it's a JSON object
        if let Some(obj) = data.as_object_mut() {
            obj.insert("message".to_string(), Value::String(message.to_string()));
        }

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/message",
            "params": {
                "level": severity.as_str(),
                "logger": "dictator",
                "data": data
            }
        });

        // Send notification (non-blocking)
        let _ = self.notif_tx.try_send(notification.to_string());

        true
    }

    /// Update minimum log level
    pub fn set_level(&self, level: Severity) {
        self.config.lock().unwrap().min_level = level;
    }

    /// Get current minimum log level
    pub fn current_level(&self) -> Severity {
        self.config.lock().unwrap().min_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Debug > Severity::Warning);
        assert!(Severity::Error < Severity::Notice);
        assert_eq!(Severity::Debug as u8, 7);
        assert_eq!(Severity::Emergency as u8, 0);
    }

    #[test]
    fn test_severity_from_string() {
        assert_eq!(Severity::from_string("debug"), Some(Severity::Debug));
        assert_eq!(Severity::from_string("DEBUG"), Some(Severity::Debug));
        assert_eq!(Severity::from_string("error"), Some(Severity::Error));
        assert_eq!(Severity::from_string("unknown"), None);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Debug.as_str(), "debug");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Error.as_str(), "error");
    }

    #[test]
    fn test_rate_limiter_allows_tokens() {
        let limiter = RateLimiter::new();
        // Should allow first tokens
        assert!(limiter.try_log());
        assert!(limiter.try_log());
    }

    #[test]
    fn test_rate_limiter_respects_capacity() {
        let limiter = RateLimiter::new();
        // Consume 100 tokens
        for _ in 0..100 {
            assert!(limiter.try_log());
        }
        // Should be depleted
        assert!(!limiter.try_log());
    }

    #[test]
    fn test_logger_config_default() {
        let cfg = LoggerConfig::default();
        assert_eq!(cfg.min_level, Severity::Warning);
    }

    #[test]
    fn test_logger_filters_by_severity() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let config = Arc::new(Mutex::new(LoggerConfig {
            min_level: Severity::Warning,
            rate_limiter: Arc::new(RateLimiter::new()),
        }));
        let logger = Logger::new(config, tx);

        // Debug (7) > Warning (4) so should be filtered
        let sent = logger.log(Severity::Debug, "Debug msg", None);
        assert!(!sent);

        // Warning (4) = Warning (4) so should be sent
        let sent = logger.log(Severity::Warning, "Warning msg", None);
        assert!(sent);

        // Error (3) < Warning (4) so should be sent (more severe)
        let sent = logger.log(Severity::Error, "Error msg", None);
        assert!(sent);
    }

    #[test]
    fn test_logger_set_level() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let config = Arc::new(Mutex::new(LoggerConfig::default()));
        let logger = Logger::new(config, tx);

        // Change to notice level
        logger.set_level(Severity::Notice);
        assert_eq!(logger.current_level(), Severity::Notice);

        // Now Debug should be sent (7 > 5)
        let sent = logger.log(Severity::Debug, "Debug msg", None);
        assert!(sent);
    }
}
