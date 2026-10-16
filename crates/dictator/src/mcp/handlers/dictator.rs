//! Dictator tool handler for auto-fixing structural issues.

use mcp_host::managers::progress::Progress;
use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::fixers::handle_kimjongrails;
use crate::mcp::linters::handle_supremecourt;
use crate::mcp::state::ServerState;
use crate::mcp::utils::{allowed_paths_within_cwd, current_dir_or_default, parse_arguments};

/// Handle dictator tool (auto-fix)
pub fn handle_dictator(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
    progress: Option<&Progress>,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct Args {
        paths: Vec<String>,
        mode: Option<String>,
        #[serde(default)]
        workspace: Option<String>,
    }

    let args: Args = match parse_arguments(&id, arguments) {
        Ok(args) => args,
        Err(response) => return *response,
    };

    // Security: dictator only works within cwd (prevents LLM from fixing /home, /etc, etc.)
    let cwd = args
        .workspace
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(current_dir_or_default);
    let allowed = match allowed_paths_within_cwd(&id, &args.paths, "dictator", &cwd) {
        Ok(allowed) => allowed,
        Err(response) => return *response,
    };

    let paths_json = serde_json::json!({"paths": allowed, "workspace": cwd.to_string_lossy()});
    let mode = args.mode.unwrap_or_else(|| "kimjongrails".to_string());

    match mode.as_str() {
        "kimjongrails" => handle_kimjongrails(id, Some(paths_json), progress),
        "supremecourt" => {
            // Run kimjongrails first, then supremecourt
            if let Some(progress) = progress {
                progress.report(0.0, Some(2.0), Some("Structural fixes".into()));
            }
            let kim_result =
                handle_kimjongrails(serde_json::json!(0), Some(paths_json.clone()), None);
            if let Some(progress) = progress {
                progress.report(1.0, Some(2.0), Some("External linters".into()));
            }
            let supreme_result = handle_supremecourt(
                serde_json::json!(0),
                Some(paths_json),
                Arc::clone(&watcher_state),
                None,
            );
            if let Some(progress) = progress {
                progress.report(2.0, Some(2.0), None);
            }

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
                jsonrpc: Default::default(),
                id: Some(id),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": combined }]
                })),
                error: None,
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: Default::default(),
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
mod tests;
