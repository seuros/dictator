//! Occupy tool handler for initializing .dictate.toml.

use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::state::ServerState;
use crate::mcp::utils::current_dir_or_default;

const DEFAULT_CONFIG: &str = include_str!("../../../templates/default.dictate.toml");

#[derive(Deserialize, Default)]
struct Args {
    #[serde(default)]
    workspace: Option<String>,
}

/// Handle `occupy` tool - initialize .dictate.toml
///
/// `arguments.workspace` overrides the process cwd, so the config is written
/// into the caller-supplied workspace instead of wherever the server happened
/// to be launched from.
pub fn handle_occupy(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) -> JsonRpcResponse {
    let args: Args = arguments
        .and_then(|a| serde_json::from_value(a).ok())
        .unwrap_or_default();

    let cwd = args
        .workspace
        .map(std::path::PathBuf::from)
        .unwrap_or_else(current_dir_or_default);

    let config_path = cwd.join(".dictate.toml");

    // Check if config already exists
    if config_path.exists() {
        return JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(id),
            result: Some(serde_json::json!({
                "content": [{ "type": "text", "text": ".dictate.toml already exists." }]
            })),
            error: None,
        };
    }

    // Write default config
    if let Err(e) = std::fs::write(&config_path, DEFAULT_CONFIG) {
        return JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(id),
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
        jsonrpc: "2.0".into(),
        id: Some(id),
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": message }]
        })),
        error: None,
    }
}
