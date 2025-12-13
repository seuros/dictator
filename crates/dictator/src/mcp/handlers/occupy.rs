//! Occupy tool handler for initializing .dictate.toml.

use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::protocol::{JsonRpcError, JsonRpcResponse};
use crate::mcp::state::ServerState;

const DEFAULT_CONFIG: &str = include_str!("../../../templates/default.dictate.toml");

/// Handle `occupy` tool - initialize .dictate.toml
pub fn handle_occupy(
    id: Value,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) -> JsonRpcResponse {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: format!("Failed to get current directory: {e}"),
                    data: None,
                }),
            };
        }
    };

    let config_path = cwd.join(".dictate.toml");

    // Check if config already exists
    if config_path.exists() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "content": [{ "type": "text", "text": ".dictate.toml already exists." }]
            })),
            error: None,
        };
    }

    // Write default config
    if let Err(e) = std::fs::write(&config_path, DEFAULT_CONFIG) {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to write config: {e}"),
                data: None,
            }),
        };
    }

    // Reload config in state
    {
        let mut state = watcher_state.lock().unwrap();
        state.config = None; // Force reload on next access
    }

    // Notify client that tool list and resources changed
    let tools_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/tools/list_changed",
        "params": {}
    });
    let _ = notif_tx.try_send(tools_notification.to_string());

    let resources_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/resources/list_changed",
        "params": {}
    });
    let _ = notif_tx.try_send(resources_notification.to_string());

    // Get client name to determine issue URL
    let client_name = {
        let state = watcher_state.lock().unwrap();
        state.client.name.clone()
    };

    let issue_url = match client_name.as_str() {
        "claude-code" => "https://github.com/anthropics/claude-code/issues",
        "codex-mcp-client" => "https://github.com/openai/codex/issues",
        _ => "https://github.com/seuros/dictator/issues",
    };

    let message = format!(
        "Created .dictate.toml with default configuration.\n\n\
         Next steps:\n\
         1. Read .dictate.toml and customize for your project\n\
         2. Tools list should refresh automatically\n\
         3. If tools don't refresh, report at {issue_url}"
    );

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": message }]
        })),
        error: None,
    }
}
