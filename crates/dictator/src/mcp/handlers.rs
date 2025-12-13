//! Individual tool handlers for stalint, dictator, and watch commands.

use camino::Utf8Path;
use dictator_core::Source;
use notify::{RecursiveMode, Watcher};
use notify_types::event::Event;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::linters::{handle_kimjongrails, handle_supremecourt, init_regime_from_config};
use super::protocol::{JsonRpcError, JsonRpcResponse};
use super::state::{DEFAULT_STALINT_LIMIT, ServerState};
use super::utils::{
    base64_decode, base64_encode, byte_to_line_col, collect_files, is_within_cwd, log_to_file,
};
use dictator_decree_abi::Span;

/// Handle stalint tool
pub fn handle_stalint(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct Args {
        #[serde(default)]
        paths: Vec<String>,
        limit: Option<usize>,
        cursor: Option<String>,
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

    // Determine paths: use provided paths, or fall back to stored paths for pagination
    let paths = if !args.paths.is_empty() {
        // New query - store paths for pagination
        {
            let mut state = watcher_state.lock().unwrap();
            state.stalint_paths.clone_from(&args.paths);
        }
        args.paths
    } else if args.cursor.is_some() {
        // Pagination - use stored paths
        let state = watcher_state.lock().unwrap();
        if state.stalint_paths.is_empty() {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message:
                        "Cursor provided but no paths stored. Provide paths to start a new query."
                            .to_string(),
                    data: None,
                }),
            };
        }
        state.stalint_paths.clone()
    } else {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Missing paths argument".to_string(),
                data: None,
            }),
        };
    };

    let limit = args.limit.unwrap_or(DEFAULT_STALINT_LIMIT);
    let offset: usize = args
        .cursor
        .and_then(|c| String::from_utf8(base64_decode(&c)).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Resolve paths to absolute (relative to server's cwd)
    let cwd = std::env::current_dir().unwrap_or_default();
    let resolved_paths: Vec<std::path::PathBuf> = paths
        .iter()
        .map(|p| {
            let path = std::path::Path::new(p);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                cwd.join(path)
            }
        })
        .collect();

    log_to_file(&format!(
        "STALINT: cwd={}, paths={:?}",
        cwd.display(),
        resolved_paths
    ));

    let regime = init_regime_from_config();

    let mut all_violations: Vec<serde_json::Value> = Vec::new();

    // Single file optimization: omit "file" field when targeting one file
    let single_file = resolved_paths.len() == 1 && resolved_paths[0].is_file();

    // Collect all files first for progress tracking
    let all_files: Vec<std::path::PathBuf> = resolved_paths
        .iter()
        .filter(|p| p.exists())
        .flat_map(|p| collect_files(p))
        .collect();

    // Start progress tracking
    let progress_token = {
        let state = watcher_state.lock().unwrap();
        state.progress_tracker.start("stalint", all_files.len() as u32)
    };

    for (file_idx, file) in all_files.iter().enumerate() {
        // Update progress
        {
            let state = watcher_state.lock().unwrap();
            state.progress_tracker.progress(&progress_token, (file_idx + 1) as u32);
        }

        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };

        // Use relative path if within cwd (saves tokens)
        let relative = file.strip_prefix(&cwd).unwrap_or(file);
        let path_str = relative.to_str().unwrap_or("<invalid>");
        let source = Source {
            path: Utf8Path::new(path_str),
            text: &text,
        };

        if let Ok(diags) = regime.enforce(&[source]) {
            for diag in &diags {
                let (line, col) = byte_to_line_col(&text, diag.span.start);
                let snippet = make_snippet(&text, &diag.span, 160);
                if single_file {
                    all_violations.push(serde_json::json!({
                        "line": line,
                        "col": col,
                        "rule": diag.rule,
                        "message": diag.message,
                        "enforced": diag.enforced,
                        "snippet": snippet,
                    }));
                } else {
                    all_violations.push(serde_json::json!({
                        "file": path_str,
                        "line": line,
                        "col": col,
                        "rule": diag.rule,
                        "message": diag.message,
                        "enforced": diag.enforced,
                        "snippet": snippet,
                    }));
                }
            }
        }
    }

    // Finish progress tracking
    {
        let state = watcher_state.lock().unwrap();
        state.progress_tracker.finish(&progress_token);
    }

    let total = all_violations.len();
    let page: Vec<_> = all_violations
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    let next_offset = offset + page.len();

    let next_cursor = if next_offset < total {
        Some(base64_encode(next_offset.to_string().as_bytes()))
    } else {
        None
    };

    let mut result = serde_json::json!({
        "content": [],
        "structuredContent": {
            "total": total,
            "returned": page.len(),
            "violations": page
        }
    });

    if let Some(cursor) = next_cursor {
        result["structuredContent"]["nextCursor"] = serde_json::json!(cursor);
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

/// Build a sanitized single-line snippet around the diagnostic span.
fn make_snippet(source: &str, span: &Span, max_len: usize) -> String {
    if source.is_empty() {
        return String::new();
    }

    let start = span.start.min(source.len());

    // Find line bounds containing the span start.
    let line_start = source[..start].rfind('\n').map_or(0, |idx| idx + 1);
    let line_end = source[start..]
        .find('\n')
        .map_or_else(|| source.len(), |off| start + off);

    let line = &source[line_start..line_end];

    // Sanitize control characters (except tab) to spaces and trim trailing whitespace.
    let mut cleaned: String = line
        .chars()
        .map(|c| if c.is_control() && c != '\t' { ' ' } else { c })
        .collect();
    cleaned.truncate(cleaned.trim_end().len());

    if cleaned.len() > max_len {
        let mut out = cleaned
            .chars()
            .take(max_len.saturating_sub(1))
            .collect::<String>();
        out.push('…');
        out
    } else {
        cleaned
    }
}

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
            let kim_result = handle_kimjongrails(serde_json::json!(0), Some(paths_json.clone()), Arc::clone(&watcher_state));
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

    // Security: stalint_watch only works within cwd
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
                    "Security: stalint_watch only operates within cwd ({}). Rejected paths: {}",
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
                jsonrpc: "2.0".to_string(),
                id,
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
        "Watching {} path(s) for changes. Will notify every 60s when violations detected.\nPaths: {}",
        watched.len(),
        watched.join(", ")
    );

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": output }]
        })),
        error: None,
    }
}

const DEFAULT_CONFIG: &str = include_str!("../../templates/default.dictate.toml");

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

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": "Created .dictate.toml with default configuration." }]
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
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": "Stopped watching for file changes." }]
        })),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_stalint_missing_arguments() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let response = handle_stalint(id, None, state);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_stalint_nonexistent_path() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let args = Some(serde_json::json!({
            "paths": ["/nonexistent/path/that/does/not/exist"]
        }));

        let response = handle_stalint(id, args, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["structuredContent"]["total"], 0);
        assert_eq!(result["structuredContent"]["returned"], 0);
    }

    #[test]
    fn test_handle_stalint_with_limit() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let id = serde_json::json!(1);
        let args = Some(serde_json::json!({
            "paths": ["/nonexistent"],
            "limit": 5
        }));

        // Should parse limit without error even if no files found
        let response = handle_stalint(id, args, state);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_handle_stalint_pagination_with_stored_paths() {
        let state = Arc::new(Mutex::new(ServerState::default()));

        // First call with paths stores them
        let id = serde_json::json!(1);
        let args = Some(serde_json::json!({
            "paths": ["/nonexistent"]
        }));
        let response = handle_stalint(id, args, Arc::clone(&state));
        assert!(response.error.is_none());

        // Second call with cursor only should use stored paths
        let id = serde_json::json!(2);
        let cursor = base64_encode(b"0");
        let args = Some(serde_json::json!({
            "cursor": cursor
        }));
        let response = handle_stalint(id, args, state);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_handle_stalint_cursor_without_stored_paths() {
        let state = Arc::new(Mutex::new(ServerState::default()));

        // Call with cursor but no paths stored should error
        let id = serde_json::json!(1);
        let cursor = base64_encode(b"10");
        let args = Some(serde_json::json!({
            "cursor": cursor
        }));
        let response = handle_stalint(id, args, state);

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("no paths stored"));
    }

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
