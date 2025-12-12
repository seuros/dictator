//! MCP progress tracking for long-running operations.
//!
//! Provides progress notifications with unique tokens, tracking operation state,
//! and automatic cleanup of stale tokens. Tokens are formatted as:
//! `"{op_type}-{unix_timestamp}-{counter}"`

use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Tracks state of an active operation
#[derive(Debug, Clone)]
pub struct OperationState {
    /// Operation type: "stalint", "dictator", "supremecourt"
    pub op_type: String,
    /// Stable operation ID (same as token)
    pub op_id: String,
    /// Total items to process
    pub total: u32,
    /// Currently processed items
    pub current: u32,
    /// When operation started
    pub start_time: Instant,
}

/// Progress tracker for long-running MCP operations
pub struct ProgressTracker {
    /// Global counter for unique progress tokens
    token_counter: Arc<AtomicU64>,
    /// Active operations: token -> state
    operations: Arc<Mutex<HashMap<String, OperationState>>>,
    /// Last cleanup time
    last_cleanup: Arc<Mutex<Instant>>,
    /// Notification channel sender
    notif_tx: tokio::sync::mpsc::Sender<String>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(notif_tx: tokio::sync::mpsc::Sender<String>) -> Self {
        Self {
            token_counter: Arc::new(AtomicU64::new(0)),
            operations: Arc::new(Mutex::new(HashMap::new())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            notif_tx,
        }
    }

    /// Generate a unique progress token: "{op_type}-{timestamp}-{counter}"
    fn generate_token(&self, op_type: &str) -> String {
        let counter = self.token_counter.fetch_add(1, Ordering::SeqCst);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}-{}-{}", op_type, timestamp, counter)
    }

    /// Start a new operation and return its unique progress token
    pub fn start(&self, op_type: &str, total: u32) -> String {
        let token = self.generate_token(op_type);
        let op_id = token.clone();

        let op_state = OperationState {
            op_type: op_type.to_string(),
            op_id,
            total,
            current: 0,
            start_time: Instant::now(),
        };

        self.operations
            .lock()
            .unwrap()
            .insert(token.clone(), op_state);

        // Send initial progress notification (0%)
        self.send_notification(&token, 0, total);

        token
    }

    /// Update progress for an operation (0..=total)
    /// Returns true if operation is still active, false if unknown/expired
    pub fn progress(&self, token: &str, current: u32) -> bool {
        let (progress, total) = {
            let mut ops = self.operations.lock().unwrap();

            if let Some(op) = ops.get_mut(token) {
                // Clamp progress to [0, total]
                op.current = current.min(op.total);
                (op.current, op.total)
            } else {
                return false;
            }
        };

        self.send_notification(token, progress, total);
        true
    }

    /// Mark operation complete (final notification at 100%)
    pub fn finish(&self, token: &str) {
        if let Some(op) = self.operations.lock().unwrap().remove(token) {
            self.send_notification(token, op.total, op.total);
        }
    }

    /// Send progress notification to client
    fn send_notification(&self, token: &str, progress: u32, total: u32) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/progress",
            "params": {
                "progressToken": token,
                "progress": progress,
                "total": total
            }
        });

        let _ = self.notif_tx.try_send(notification.to_string());
    }

    /// Clean up stale operations (older than 10 minutes)
    /// Call periodically from watcher loop
    pub fn cleanup_stale(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        let now = Instant::now();

        // Only cleanup every 60 seconds
        if now.duration_since(*last_cleanup) < Duration::from_secs(60) {
            return;
        }

        *last_cleanup = now;

        let timeout = Duration::from_secs(600); // 10 minutes
        self.operations
            .lock()
            .unwrap()
            .retain(|_token, op| now.duration_since(op.start_time) < timeout);
    }

    /// Get current progress of an operation (for debugging)
    #[cfg(test)]
    fn get_progress(&self, token: &str) -> Option<(u32, u32)> {
        self.operations
            .lock()
            .unwrap()
            .get(token)
            .map(|op| (op.current, op.total))
    }

    /// Check if operation is tracked
    #[cfg(test)]
    fn has_operation(&self, token: &str) -> bool {
        self.operations.lock().unwrap().contains_key(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tracker() -> ProgressTracker {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        ProgressTracker::new(tx)
    }

    #[test]
    fn test_token_format() {
        let tracker = create_tracker();
        let token = tracker.generate_token("stalint");

        // Format: "{op_type}-{timestamp}-{counter}"
        let parts: Vec<&str> = token.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "stalint");
        // parts[1] is unix timestamp
        assert!(parts[1].parse::<u64>().is_ok());
        // parts[2] is counter
        assert!(parts[2].parse::<u64>().is_ok());
    }

    #[test]
    fn test_token_uniqueness() {
        let tracker = create_tracker();
        let token1 = tracker.generate_token("stalint");
        let token2 = tracker.generate_token("stalint");

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_start_registers_operation() {
        let tracker = create_tracker();
        let token = tracker.start("stalint", 100);

        assert!(tracker.has_operation(&token));
        let (current, total) = tracker.get_progress(&token).unwrap();
        assert_eq!(current, 0);
        assert_eq!(total, 100);
    }

    #[test]
    fn test_progress_updates() {
        let tracker = create_tracker();
        let token = tracker.start("dictator", 50);

        // Update to 50%
        assert!(tracker.progress(&token, 25));
        let (current, total) = tracker.get_progress(&token).unwrap();
        assert_eq!(current, 25);
        assert_eq!(total, 50);

        // Update to 100%
        assert!(tracker.progress(&token, 50));
        let (current, total) = tracker.get_progress(&token).unwrap();
        assert_eq!(current, 50);
        assert_eq!(total, 50);
    }

    #[test]
    fn test_progress_clamps_to_total() {
        let tracker = create_tracker();
        let token = tracker.start("supremecourt", 10);

        // Try to set beyond total
        assert!(tracker.progress(&token, 99));
        let (current, total) = tracker.get_progress(&token).unwrap();
        assert_eq!(current, 10); // Clamped to total
        assert_eq!(total, 10);
    }

    #[test]
    fn test_finish_removes_operation() {
        let tracker = create_tracker();
        let token = tracker.start("stalint", 100);

        assert!(tracker.has_operation(&token));
        tracker.finish(&token);
        assert!(!tracker.has_operation(&token));
    }

    #[test]
    fn test_progress_unknown_token() {
        let tracker = create_tracker();

        // Unknown token should return false
        assert!(!tracker.progress("unknown-token", 50));
    }

    #[test]
    fn test_multiple_concurrent_operations() {
        let tracker = create_tracker();
        let token1 = tracker.start("stalint", 100);
        let token2 = tracker.start("dictator", 50);

        assert!(tracker.has_operation(&token1));
        assert!(tracker.has_operation(&token2));

        // Update both independently
        assert!(tracker.progress(&token1, 50));
        assert!(tracker.progress(&token2, 25));

        let (c1, t1) = tracker.get_progress(&token1).unwrap();
        let (c2, t2) = tracker.get_progress(&token2).unwrap();

        assert_eq!((c1, t1), (50, 100));
        assert_eq!((c2, t2), (25, 50));

        // Finish one
        tracker.finish(&token1);
        assert!(!tracker.has_operation(&token1));
        assert!(tracker.has_operation(&token2));
    }
}
