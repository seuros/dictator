//! Logging level request handler for MCP protocol.

use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::logging::Severity;
use crate::mcp::protocol::{JsonRpcError, JsonRpcResponse};
use crate::mcp::state::ServerState;

#[derive(Deserialize)]
struct SetLevelParams {
    level: String,
}

/// Handle logging/setLevel request - set minimum log severity
pub fn handle_logging_set_level(
    id: Value,
    params: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
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

    let set_level_params: SetLevelParams = match serde_json::from_value(params) {
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

    // Parse severity level
    let Some(severity) = Severity::from_string(&set_level_params.level) else {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: format!("Unknown level: {}", set_level_params.level),
                data: None,
            }),
        };
    };

    // Update logger config in state
    {
        let state = watcher_state.lock().unwrap();
        state.logger_config.lock().unwrap().min_level = severity;
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "level": set_level_params.level
        })),
        error: None,
    }
}
