//! Stalint tool handler for structural linting.

use camino::Utf8Path;
use dictator_core::Source;
use mcp_host::managers::progress::Progress;
use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use mcp_host::utils::paginate;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::regime::init_regime_from_config;
use crate::mcp::state::{DEFAULT_STALINT_LIMIT, ServerState};
use crate::mcp::utils::{
    byte_to_line_col, collect_files, current_dir_or_default, log_to_file, make_snippet,
};

/// Handle stalint tool
pub fn handle_stalint(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
    progress: Option<&Progress>,
) -> JsonRpcResponse {
    #[derive(Deserialize, Default)]
    struct Args {
        #[serde(default)]
        paths: Vec<String>,
        limit: Option<usize>,
        cursor: Option<String>,
        #[serde(default)]
        workspace: Option<String>,
    }

    let args: Args = arguments
        .and_then(|a| serde_json::from_value(a).ok())
        .unwrap_or_default();

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
                jsonrpc: Default::default(),
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

    // Resolve paths to absolute (relative to server's cwd)
    let cwd = args
        .workspace
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(current_dir_or_default);
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

    let regime = init_regime_from_config(Some(&cwd));

    let mut all_violations: Vec<serde_json::Value> = Vec::new();

    // Single file optimization: omit "file" field when targeting one file
    let single_file = resolved_paths.len() == 1 && resolved_paths[0].is_file();

    // Collect all files first for progress tracking
    let mut all_files: Vec<std::path::PathBuf> = resolved_paths
        .iter()
        .filter(|p| p.exists())
        .flat_map(|p| collect_files(p))
        .collect();
    // Each page is a fresh scan; neither directory walks nor decrees promise ordering.
    all_files.sort_unstable();

    // Start progress tracking
    let total = f64::from(u32::try_from(all_files.len()).unwrap_or(u32::MAX));
    if let Some(progress) = progress {
        progress.report(0.0, Some(total), Some("stalint".into()));
    }

    for (file_idx, file) in all_files.iter().enumerate() {
        // Update progress
        if let Some(progress) = progress {
            let current = f64::from(u32::try_from(file_idx + 1).unwrap_or(u32::MAX));
            progress.report(current, Some(total), None);
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

        if let Ok(mut diags) = regime.enforce(&[source]) {
            diags.sort_unstable_by(|a, b| {
                (a.span.start, a.span.end, &a.rule, &a.message, a.enforced).cmp(&(
                    b.span.start,
                    b.span.end,
                    &b.rule,
                    &b.message,
                    b.enforced,
                ))
            });
            for diag in &diags {
                let (line, col) = byte_to_line_col(&text, diag.span.start);
                let snippet = if diag.rule == dictator_core::classified::CLASSIFIED_RULE {
                    String::new()
                } else {
                    make_snippet(&text, &diag.span, 160)
                };
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

    let total = all_violations.len();
    {
        let mut state = watcher_state.lock().unwrap();
        state.record_violations(total);
    }
    let page = match paginate(&all_violations, args.cursor.as_deref(), limit) {
        Ok(page) => page,
        Err(error) => {
            return JsonRpcResponse {
                jsonrpc: Default::default(),
                id: Some(id),
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: error.to_string(),
                    data: None,
                }),
            };
        }
    };

    let mut result = serde_json::json!({
        "content": [],
        "structuredContent": {
            "total": total,
            "returned": page.items.len(),
            "violations": page.items
        }
    });

    if let Some(cursor) = page.next_cursor {
        result["structuredContent"]["nextCursor"] = serde_json::json!(cursor);
    }

    JsonRpcResponse {
        jsonrpc: Default::default(),
        id: Some(id),
        result: Some(result),
        error: None,
    }
}

#[cfg(test)]
mod tests;
