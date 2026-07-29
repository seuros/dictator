//! Watch/unwatch tool handlers for file monitoring.

use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use notify::{RecursiveMode, Watcher};
use notify_types::event::Event;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::mcp::state::ServerState;
use crate::mcp::utils::{allowed_paths_within_cwd, parse_arguments};

/// Handle `stalint_watch` tool
pub fn handle_stalint_watch(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct Args {
        paths: Vec<String>,
    }

    let args: Args = match parse_arguments(&id, arguments) {
        Ok(args) => args,
        Err(response) => return *response,
    };

    // Security: stalint_watch only works within cwd
    let allowed = match allowed_paths_within_cwd(&id, &args.paths, "stalint_watch") {
        Ok(allowed) => allowed,
        Err(response) => return *response,
    };

    // Set up file watcher
    let watcher_state_clone = Arc::clone(&watcher_state);
    let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res
            && (event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove())
            && let Ok(mut state) = watcher_state_clone.lock()
        {
            state.dirty = true;
        }
    });

    let mut watcher = match watcher {
        Ok(w) => w,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: Some(id),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: format!("Failed to create watcher: {e}"),
                    data: None,
                }),
            };
        }
    };

    // Watch all allowed paths
    let mut watched = Vec::new();
    for path in &allowed {
        let p = std::path::Path::new(path);
        if p.exists() {
            if let Err(e) = watcher.watch(p, RecursiveMode::Recursive) {
                tracing::warn!("Failed to watch {}: {}", path, e);
            } else {
                watched.push(path.clone());
            }
        }
    }

    // Update shared state
    {
        let mut state = watcher_state.lock().unwrap();
        state.paths.extend(watched.iter().cloned());
        state.watcher = Some(watcher);
        state.dirty = true;
        state.last_check = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
        state.is_watching = true;
    }

    // Notify client that tool list changed (stalint_watch -> stalint_unwatch)
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/tools/list_changed",
        "params": {}
    });
    let _ = notif_tx.try_send(notification.to_string());

    let output = format!(
        "Watching {} path(s). Notifying every 60s on violations.\nPaths: {}",
        watched.len(),
        watched.join(", ")
    );

    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: Some(id),
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": output }]
        })),
        error: None,
    }
}

/// Handle `stalint_unwatch` tool
pub fn handle_stalint_unwatch(
    id: Value,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) -> JsonRpcResponse {
    // Clear watcher state
    {
        let mut state = watcher_state.lock().unwrap();
        state.paths.clear();
        state.watcher = None;
        state.is_watching = false;
        state.dirty = false;
    }

    // Notify client that tool list changed (stalint_unwatch -> stalint_watch)
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/tools/list_changed",
        "params": {}
    });
    let _ = notif_tx.try_send(notification.to_string());

    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: Some(id),
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": "Stopped watching for file changes." }]
        })),
        error: None,
    }
}
