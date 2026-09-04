//! Stalint tool handler for structural linting.

use camino::Utf8Path;
use dictator_core::Source;
use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::regime::init_regime_from_config;
use crate::mcp::state::{DEFAULT_STALINT_LIMIT, ServerState};
use crate::mcp::utils::{
    base64_decode, base64_encode, byte_to_line_col, collect_files, current_dir_or_default,
    log_to_file, make_snippet,
};

/// Handle stalint tool
pub fn handle_stalint(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    #[derive(Deserialize, Default)]
    struct Args {
        #[serde(default)]
        paths: Vec<String>,
        limit: Option<usize>,
        cursor: Option<String>,
    }

    let args: Args = arguments.and_then(|a| serde_json::from_value(a).ok()).unwrap_or_default();

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
                jsonrpc: "2.0".into(),
                id: Some(id),
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
        // Default to current working directory
        vec![".".to_string()]
    };

    let limit = args.limit.unwrap_or(DEFAULT_STALINT_LIMIT);
    let offset: usize = args
        .cursor
        .and_then(|c| String::from_utf8(base64_decode(&c)).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Resolve paths to absolute (relative to server's cwd)
    let cwd = current_dir_or_default();
    let resolved_paths: Vec<std::path::PathBuf> = paths
        .iter()
        .map(|p| {
            let path = std::path::Path::new(p);
            if path.is_absolute() { path.to_path_buf() } else { cwd.join(path) }
        })
        .collect();

    log_to_file(&format!("STALINT: cwd={}, paths={:?}", cwd.display(), resolved_paths));

    let regime = init_regime_from_config();

    let mut all_violations: Vec<serde_json::Value> = Vec::new();

    // Single file optimization: omit "file" field when targeting one file
    let single_file = resolved_paths.len() == 1 && resolved_paths[0].is_file();

    // Collect all files first for progress tracking
    let all_files: Vec<std::path::PathBuf> =
        resolved_paths.iter().filter(|p| p.exists()).flat_map(|p| collect_files(p)).collect();

    // Start progress tracking
    let progress_token = {
        let state = watcher_state.lock().unwrap();
        let total = u32::try_from(all_files.len()).unwrap_or(u32::MAX);
        state.progress_tracker.start("stalint", total)
    };

    for (file_idx, file) in all_files.iter().enumerate() {
        // Update progress
        {
            let state = watcher_state.lock().unwrap();
            let current = u32::try_from(file_idx + 1).unwrap_or(u32::MAX);
            state.progress_tracker.progress(&progress_token, current);
        }

        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };

        // Use relative path if within cwd (saves tokens)
        let relative = file.strip_prefix(&cwd).unwrap_or(file);
        let path_str = relative.to_str().unwrap_or("<invalid>");
        let source = Source { path: Utf8Path::new(path_str), text: &text };

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
    let page: Vec<_> = all_violations.into_iter().skip(offset).take(limit).collect();
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

    JsonRpcResponse { jsonrpc: "2.0".into(), id: Some(id), result: Some(result), error: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::utils::base64_encode;

    #[test]
    fn stalint_handler_covers_defaults_paths_and_pagination() {
        let state = || Arc::new(Mutex::new(ServerState::default()));

        // No arguments defaults to cwd and succeeds.
        let response = handle_stalint(serde_json::json!(1), None, state());
        assert!(response.error.is_none());
        assert!(response.result.unwrap()["structuredContent"]["total"].is_number());

        // Nonexistent path lints nothing, and a limit parses fine alongside it.
        let args = Some(serde_json::json!({
            "paths": ["/nonexistent/path/that/does/not/exist"],
            "limit": 5
        }));
        let response = handle_stalint(serde_json::json!(1), args, state());
        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["structuredContent"]["total"], 0);
        assert_eq!(result["structuredContent"]["returned"], 0);

        // Paths from a first call are stored and reused by a cursor-only follow-up.
        let shared = state();
        let args = Some(serde_json::json!({"paths": ["/nonexistent"]}));
        let response = handle_stalint(serde_json::json!(1), args, Arc::clone(&shared));
        assert!(response.error.is_none());
        let args = Some(serde_json::json!({"cursor": base64_encode(b"0")}));
        let response = handle_stalint(serde_json::json!(2), args, shared);
        assert!(response.error.is_none());

        // A cursor without stored paths is an invalid-params error.
        let args = Some(serde_json::json!({"cursor": base64_encode(b"10")}));
        let response = handle_stalint(serde_json::json!(1), args, state());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("no paths stored"));
    }
}
