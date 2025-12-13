//! Dictator tool handler for auto-fixing structural issues.

use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::fixers::handle_kimjongrails;
use crate::mcp::linters::handle_supremecourt;
use crate::mcp::protocol::{JsonRpcError, JsonRpcResponse};
use crate::mcp::state::ServerState;
use crate::mcp::utils::is_within_cwd;

/// Handle dictator tool (auto-fix)
pub fn handle_dictator(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct Args {
        paths: Vec<String>,
        mode: Option<String>,
    }

    let args: Args = match arguments.and_then(|a| serde_json::from_value(a).ok()) {
        Some(a) => a,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing or invalid arguments".to_string(),
                    data: None,
                }),
            };
        }
    };

    // Security: dictator only works within cwd (prevents LLM from fixing /home, /etc, etc.)
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut rejected: Vec<String> = Vec::new();
    let mut allowed: Vec<String> = Vec::new();

    for path in &args.paths {
        let p = std::path::Path::new(path);
        if is_within_cwd(p, &cwd) {
            allowed.push(path.clone());
        } else {
            rejected.push(path.clone());
        }
    }

    if !rejected.is_empty() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: format!(
                    "Security: dictator only operates within cwd ({}). Rejected paths: {}",
                    cwd.display(),
                    rejected.join(", ")
                ),
                data: None,
            }),
        };
    }

    if allowed.is_empty() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "No valid paths provided".to_string(),
                data: None,
            }),
        };
    }

    let paths_json = serde_json::json!({"paths": allowed});
    let mode = args.mode.unwrap_or_else(|| "kimjongrails".to_string());

    match mode.as_str() {
        "kimjongrails" => handle_kimjongrails(id, Some(paths_json), Arc::clone(&watcher_state)),
        "supremecourt" => {
            // Run kimjongrails first, then supremecourt
            let kim_result = handle_kimjongrails(
                serde_json::json!(0),
                Some(paths_json.clone()),
                Arc::clone(&watcher_state),
            );
            let supreme_result = handle_supremecourt(
                serde_json::json!(0),
                Some(paths_json),
                Arc::clone(&watcher_state),
            );

            // Combine outputs
            let kim_text = kim_result
                .result
                .as_ref()
                .and_then(|r| r.get("content"))
                .and_then(|c| c.get(0))
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            let supreme_text = supreme_result
                .result
                .as_ref()
                .and_then(|r| r.get("content"))
                .and_then(|c| c.get(0))
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("");

            let combined = format!("{kim_text}\n\n{supreme_text}");
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": combined }]
                })),
                error: None,
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: format!("Unknown mode: {mode}. Use kimjongrails or supremecourt"),
                data: None,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_dictator_missing_arguments() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let response = handle_dictator(id, None, state);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_dictator_unknown_mode() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        // Use relative path (within cwd) to pass security check
        let args = Some(serde_json::json!({
            "paths": ["sandbox"],
            "mode": "unknown_mode"
        }));

        let response = handle_dictator(id, args, state);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Unknown mode"));
    }

    #[test]
    fn test_handle_dictator_default_mode() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        // Use relative path (within cwd) to pass security check
        let args = Some(serde_json::json!({
            "paths": ["nonexistent_but_within_cwd"]
        }));

        // Should use kimjongrails as default mode
        let response = handle_dictator(id, args, state);
        // Even with nonexistent path, should not error on mode parsing
        assert!(response.error.is_none());
    }

    #[test]
    fn test_handle_dictator_rejects_outside_cwd() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let args = Some(serde_json::json!({
            "paths": ["/tmp", "/etc"]
        }));

        let response = handle_dictator(id, args, state);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Security"));
        assert!(error.message.contains("only operates within cwd"));
    }
}
