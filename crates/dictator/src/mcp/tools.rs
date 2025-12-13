//! MCP tool definitions and routing.

use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use super::handlers::{
    handle_dictator, handle_occupy, handle_stalint, handle_stalint_unwatch, handle_stalint_watch,
};
use super::protocol::{
    ClientInfo, Implementation, JsonRpcError, JsonRpcResponse, MIN_CLIENT_VERSIONS,
};
use super::state::ServerState;
use super::utils::{command_available, get_log_path, is_git_repo, log_to_file};

#[derive(Deserialize)]
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    #[allow(dead_code)]
    protocol_version: String,
    #[serde(rename = "clientInfo")]
    client_info: Implementation,
    #[allow(dead_code)]
    capabilities: Value,
}

#[derive(Deserialize)]
struct CallToolParams {
    name: String,
    arguments: Option<Value>,
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

/// Handle tools/list request
pub fn handle_list_tools(id: Value, watcher_state: Arc<Mutex<ServerState>>) -> JsonRpcResponse {
    tracing::info!("Handling tools/list");

    let (is_watching, can_write, config_exists) = {
        let mut state = watcher_state.lock().unwrap();
        state.ensure_config_loaded();
        (state.is_watching, state.can_write, state.config.is_some())
    };

    // No config? Only show occupy tool
    if !config_exists && can_write {
        let tools = serde_json::json!({
            "tools": [serde_json::json!({
                "name": "occupy",
                "title": "Initialize Config",
                "description": "Initialize .dictate.toml with default configuration.",
                "annotations": {
                    "title": "Initialize Config",
                    "readOnlyHint": false,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false
                },
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            })]
        });

        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(tools),
            error: None,
        };
    }

    // Watch tool changes based on state
    let watch_tool = if is_watching {
        serde_json::json!({
            "name": "stalint_unwatch",
            "title": "Stop Watcher",
            "description": "Stop watching for file changes.",
            "annotations": {
                "title": "Stop Watcher",
                "readOnlyHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        })
    } else {
        serde_json::json!({
            "name": "stalint_watch",
            "title": "File Watcher",
            "description": "Watch paths for file changes. Runs stalint every 60s when changes detected and sends notifications with violations.",
            "annotations": {
                "title": "File Watcher",
                "readOnlyHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File or directory paths to watch"
                    }
                },
                "required": ["paths"]
            }
        })
    };

    // Build tools list dynamically based on capabilities
    let mut tool_list = vec![serde_json::json!({
        "name": "stalint",
        "title": "Structural Linter",
        "description": "Check files for structural violations (trailing whitespace, tabs/spaces, line endings, file size). Read-only - returns diagnostics without modifying files.",
        "annotations": {
            "title": "Structural Linter",
            "readOnlyHint": true,
            "openWorldHint": false
        },
        "inputSchema": {
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "File or directory paths to check"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max violations to return (default 10)"
                },
                "cursor": {
                    "type": "string",
                    "description": "Pagination cursor from previous response"
                }
            },
            "required": ["paths"]
        }
    })];

    // Only include write tools if conditions are met:
    // 1. Can write (not in read-only sandbox)
    // 2. Inside a git repository (safety check)
    let in_git_repo = is_git_repo();
    if can_write && in_git_repo {
        // Check which configured linters are available
        let mut state = watcher_state.lock().unwrap();
        state.ensure_config_loaded();
        let has_supreme = state.config.as_ref().is_some_and(|cfg| {
            cfg.decree.values().any(|decree| {
                decree
                    .linter
                    .as_ref()
                    .is_some_and(|linter| command_available(&linter.command))
            })
        });
        drop(state);

        // Build mode enum and description based on available tools
        let (modes, mode_desc) = if has_supreme {
            (
                vec!["kimjongrails", "supremecourt"],
                "kimjongrails (default): basic fixes. supremecourt: basic + configured external linters",
            )
        } else {
            (
                vec!["kimjongrails"],
                "kimjongrails: basic structural fixes (whitespace, newlines, line endings)",
            )
        };

        tool_list.push(serde_json::json!({
            "name": "dictator",
            "title": "Auto-Fixer",
            "description": if has_supreme {
                "Auto-fix structural issues. Default mode fixes whitespace/newlines. Mode 'supremecourt' also runs configured external linters."
            } else {
                "Auto-fix structural issues (whitespace, newlines, line endings). Requires git repository."
            },
            "annotations": {
                "title": "Auto-Fixer",
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File or directory paths to fix"
                    },
                    "mode": {
                        "type": "string",
                        "enum": modes,
                        "description": mode_desc
                    }
                },
                "required": ["paths"]
            }
        }));
    }

    tool_list.push(watch_tool);

    let tools = serde_json::json!({ "tools": tool_list });

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(tools),
        error: None,
    }
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

/// Handle logging/setLevel request - set minimum log severity
pub fn handle_logging_set_level(
    id: Value,
    params: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    use super::logging::Severity;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct SetLevelParams {
        level: String,
    }

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

    #[test]
    fn test_handle_list_tools_not_watching() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        {
            let mut s = state.lock().unwrap();
            s.config = Some(super::super::state::DictateConfig::default());
        }
        let id = serde_json::json!(1);

        let response = handle_list_tools(id, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // Always has stalint and stalint_watch
        // dictator only if in git repo
        assert!(tools.len() >= 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"stalint"));
        assert!(tool_names.contains(&"stalint_watch"));
        assert!(!tool_names.contains(&"stalint_unwatch"));
        // dictator present only if .git exists
        if is_git_repo() {
            assert!(tool_names.contains(&"dictator"));
        }
    }

    #[test]
    fn test_handle_list_tools_watching() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        {
            let mut s = state.lock().unwrap();
            s.is_watching = true;
            s.config = Some(super::super::state::DictateConfig::default());
        }
        let id = serde_json::json!(1);

        let response = handle_list_tools(id, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"stalint_unwatch"));
        assert!(!tool_names.contains(&"stalint_watch"));
    }

    #[test]
    fn test_handle_list_tools_annotations() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        {
            let mut s = state.lock().unwrap();
            s.config = Some(super::super::state::DictateConfig::default());
        }
        let id = serde_json::json!(1);

        let response = handle_list_tools(id, state);

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // stalint should be read-only
        let stalint = tools.iter().find(|t| t["name"] == "stalint").unwrap();
        assert_eq!(stalint["annotations"]["readOnlyHint"], true);

        // dictator annotations (only if in git repo)
        if is_git_repo() {
            let dictator = tools.iter().find(|t| t["name"] == "dictator").unwrap();
            assert_eq!(dictator["annotations"]["readOnlyHint"], false);
            assert_eq!(dictator["annotations"]["destructiveHint"], true);
            assert_eq!(dictator["annotations"]["idempotentHint"], true);
        }
    }

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
