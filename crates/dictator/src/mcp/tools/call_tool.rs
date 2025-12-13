//! Tool call routing for MCP protocol.

use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::handlers::{
    handle_dictator, handle_occupy, handle_stalint, handle_stalint_unwatch, handle_stalint_watch,
};
use crate::mcp::protocol::{JsonRpcError, JsonRpcResponse};
use crate::mcp::state::ServerState;

#[derive(Deserialize)]
struct CallToolParams {
    name: String,
    arguments: Option<Value>,
}

/// Handle tools/call request - routes to specific tool handlers
pub fn handle_call_tool(
    id: Value,
    params: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) -> JsonRpcResponse {
    tracing::info!("Handling tools/call");

    let Some(params) = params else {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Missing params".to_string(),
                data: None,
            }),
        };
    };

    let call_params: CallToolParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Invalid params: {e}"),
                    data: None,
                }),
            };
        }
    };

    match call_params.name.as_str() {
        "stalint" => handle_stalint(id, call_params.arguments, watcher_state),
        "dictator" => handle_dictator(id, call_params.arguments, watcher_state),
        "stalint_watch" => handle_stalint_watch(id, call_params.arguments, watcher_state, notif_tx),
        "stalint_unwatch" => handle_stalint_unwatch(id, watcher_state, notif_tx),
        "occupy" => handle_occupy(id, watcher_state, notif_tx),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Unknown tool: {}", call_params.name),
                data: None,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_call_tool_missing_params() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let id = serde_json::json!(1);

        let response = handle_call_tool(id, None, state, tx);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_call_tool_unknown_tool() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let id = serde_json::json!(1);
        let params = Some(serde_json::json!({
            "name": "nonexistent_tool",
            "arguments": {}
        }));

        let response = handle_call_tool(id, params, state, tx);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Unknown tool"));
    }
}
