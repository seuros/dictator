//! Initialize request handler for MCP protocol.

use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::protocol::{
    ClientInfo, Implementation, JsonRpcError, JsonRpcResponse, MIN_CLIENT_VERSIONS,
};
use crate::mcp::state::ServerState;
use crate::mcp::utils::{get_log_path, log_to_file};

#[derive(Deserialize)]
pub(super) struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    #[allow(dead_code)]
    pub protocol_version: String,
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
    #[allow(dead_code)]
    pub capabilities: Value,
}

/// Handle JSON-RPC initialize request
pub fn handle_initialize(
    id: Value,
    params: Option<Value>,
    server_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    tracing::info!("Handling initialize");

    // Parse client's initialize request
    let init_params: InitializeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
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
        },
        None => {
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
        }
    };

    // Build client info
    let client = ClientInfo {
        name: init_params.client_info.name.clone(),
        version: init_params.client_info.version.clone(),
    };

    log_to_file(&format!(
        "INIT: client = {} v{}",
        client.name, client.version
    ));

    // Check minimum version
    if !client.is_supported() {
        let min_version = MIN_CLIENT_VERSIONS
            .iter()
            .find(|(n, _)| *n == client.name)
            .map_or("unknown", |(_, v)| *v);

        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: format!(
                    "Client {} v{} is too old. Minimum required: v{}. Update your client.",
                    client.name, client.version, min_version
                ),
                data: None,
            }),
        };
    }

    // Store client info and check config
    let config_exists = {
        let mut state = server_state.lock().unwrap();
        state.client = client;
        state.ensure_config_loaded();
        state.config.is_some()
    };

    // Write client info to file for debugging
    let client_log = format!(
        "{} v{}\n",
        init_params.client_info.name, init_params.client_info.version
    );
    let client_file = get_log_path("client.txt");
    let _ = std::fs::write(&client_file, &client_log);

    tracing::info!(
        "Client connected: {} v{}",
        init_params.client_info.name,
        init_params.client_info.version
    );

    // Build instructions based on config presence
    let instructions = if config_exists {
        "Dictator enforces structural code hygiene \
         (whitespace, indentation, line endings, file size)."
    } else {
        "Dictator enforces structural code hygiene. No .dictate.toml found - \
         run 'occupy' tool to initialize, then customize for your project."
    };

    // Respond with server capabilities
    let result = serde_json::json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {
            "tools": {
                "listChanged": true
            },
            "resources": {},
            "logging": {},
            "experimental": {
                "codex/sandbox-state": { "version": "1.0.0" }
            }
        },
        "serverInfo": {
            "name": "dictator",
            "title": "Dictator Structural Linter",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": instructions
    });

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_init_params(name: &str, version: &str) -> Value {
        serde_json::json!({
            "protocolVersion": "2025-06-18",
            "clientInfo": {
                "name": name,
                "version": version
            },
            "capabilities": {}
        })
    }

    #[test]
    fn test_handle_initialize_valid_client() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let params = make_init_params("claude-code", "2.0.56");

        let response = handle_initialize(id, Some(params), state);

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-06-18");
        assert_eq!(result["serverInfo"]["name"], "dictator");
    }

    #[test]
    fn test_handle_initialize_old_client_rejected() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let params = make_init_params("claude-code", "2.0.55");

        let response = handle_initialize(id, Some(params), state);

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert!(error.message.contains("too old"));
        assert!(error.message.contains("2.0.56"));
    }

    #[test]
    fn test_handle_initialize_unknown_client_allowed() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let params = make_init_params("some-new-client", "0.1.0");

        let response = handle_initialize(id, Some(params), state);

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_handle_initialize_missing_params() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);

        let response = handle_initialize(id, None, state);

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_initialize_invalid_params() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let params = Some(serde_json::json!({"invalid": "data"}));

        let response = handle_initialize(id, params, state);

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }
}
