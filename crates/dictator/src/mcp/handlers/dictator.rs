//! Dictator tool handler for auto-fixing structural issues.

use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::fixers::handle_kimjongrails;
use crate::mcp::linters::handle_supremecourt;
use crate::mcp::state::ServerState;
use crate::mcp::utils::{allowed_paths_within_cwd, parse_arguments};

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

    let args: Args = match parse_arguments(&id, arguments) {
        Ok(args) => args,
        Err(response) => return *response,
    };

    // Security: dictator only works within cwd (prevents LLM from fixing /home, /etc, etc.)
    let allowed = match allowed_paths_within_cwd(&id, &args.paths, "dictator") {
        Ok(allowed) => allowed,
        Err(response) => return *response,
    };

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
                jsonrpc: "2.0".into(),
                id: Some(id),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": combined }]
                })),
                error: None,
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(id),
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
    fn dictator_handler_validates_arguments_and_scope() {
        let state = || Arc::new(Mutex::new(ServerState::default()));

        // Missing arguments is invalid params.
        let response = handle_dictator(serde_json::json!(1), None, state());
        assert_eq!(response.error.unwrap().code, -32602);

        // Unknown mode is rejected (relative path passes the cwd security check).
        let args = Some(serde_json::json!({"paths": ["sandbox"], "mode": "unknown_mode"}));
        let response = handle_dictator(serde_json::json!(1), args, state());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Unknown mode"));

        // Default mode (kimjongrails) parses even for a nonexistent relative path.
        let args = Some(serde_json::json!({"paths": ["nonexistent_but_within_cwd"]}));
        let response = handle_dictator(serde_json::json!(1), args, state());
        assert!(response.error.is_none());

        // Absolute paths outside cwd are refused.
        let args = Some(serde_json::json!({"paths": ["/tmp", "/etc"]}));
        let response = handle_dictator(serde_json::json!(1), args, state());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Security"));
        assert!(error.message.contains("only operates within cwd"));
    }
}
