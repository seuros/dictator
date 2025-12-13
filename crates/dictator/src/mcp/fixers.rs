//! Auto-fix handlers for MCP tools.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use super::protocol::{JsonRpcError, JsonRpcResponse};
use super::state::ServerState;
use super::utils::collect_files;

/// Handle kimjongrails auto-fix (whitespace, newlines, line endings)
pub fn handle_kimjongrails(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
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

    let mut log_output = String::new();
    let mut fixed_count = 0;
    let mut rule_counts: HashMap<&str, usize> = HashMap::new();

    // Collect all files first for progress tracking
    let all_files: Vec<std::path::PathBuf> = args
        .paths
        .iter()
        .map(std::path::Path::new)
        .filter(|p| p.exists())
        .flat_map(collect_files)
        .collect();

    // Start progress tracking
    let progress_token = {
        let state = watcher_state.lock().unwrap();
        let total = u32::try_from(all_files.len()).unwrap_or(u32::MAX);
        state.progress_tracker.start("dictator", total)
    };

    for (file_idx, file) in all_files.iter().enumerate() {
        // Update progress
        {
            let state = watcher_state.lock().unwrap();
            let current = u32::try_from(file_idx + 1).unwrap_or(u32::MAX);
            state.progress_tracker.progress(&progress_token, current);
        }

        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => {
                let _ = writeln!(log_output, "! Cannot read {}: {}", file.display(), e);
                continue;
            }
        };

        let (fixed, changes) = apply_fixes(&text);

        if !changes.is_empty() && fixed != text {
            if let Err(e) = std::fs::write(file, &fixed) {
                let _ = writeln!(log_output, "! Cannot write {}: {}", file.display(), e);
            } else {
                fixed_count += 1;
                let _ = writeln!(log_output, "* {} ({})", file.display(), changes.join(", "));

                // Track counts by rule
                for change in &changes {
                    let rule = match change.as_str() {
                        "trailing whitespace" => "supreme/trailing-whitespace",
                        "final newline" => "supreme/missing-final-newline",
                        "CRLF->LF" => "supreme/crlf",
                        _ => "supreme/unknown",
                    };
                    *rule_counts.entry(rule).or_default() += 1;
                }
            }
        }
    }

    // Finish progress tracking
    {
        let state = watcher_state.lock().unwrap();
        state.progress_tracker.finish(&progress_token);
    }

    let output = build_summary(fixed_count, &rule_counts, &log_output);

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": output }]
        })),
        error: None,
    }
}

/// Apply whitespace fixes to text, returning fixed text and list of changes
fn apply_fixes(text: &str) -> (String, Vec<String>) {
    let mut fixed = text.to_string();
    let mut changes = Vec::new();

    // Fix trailing whitespace
    let lines: Vec<&str> = fixed.lines().collect();
    let trimmed: Vec<String> = lines.iter().map(|l| l.trim_end().to_string()).collect();
    if lines.iter().zip(trimmed.iter()).any(|(a, b)| *a != b) {
        fixed = trimmed.join("\n");
        changes.push("trailing whitespace".to_string());
    }

    // Ensure final newline
    if !fixed.ends_with('\n') {
        fixed.push('\n');
        changes.push("final newline".to_string());
    }

    // Normalize line endings to LF
    if fixed.contains("\r\n") {
        fixed = fixed.replace("\r\n", "\n");
        changes.push("CRLF->LF".to_string());
    }

    (fixed, changes)
}

/// Build summary output for fixes
fn build_summary(
    fixed_count: usize,
    rule_counts: &HashMap<&str, usize>,
    log_output: &str,
) -> String {
    if fixed_count == 0 {
        return "No fixes needed".to_string();
    }

    // Write log file with details
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let log_path = format!("/tmp/dictator-fixes-{timestamp}.log");

    let log_written = std::fs::write(&log_path, log_output).is_ok();

    // Build summary grouped by rule
    let mut summary = format!("Fixed {fixed_count} files:\n");
    let mut rules: Vec<_> = rule_counts.iter().collect();
    rules.sort_by_key(|(rule, _)| *rule);
    for (rule, count) in rules {
        let _ = writeln!(summary, "  {rule}: {count}");
    }
    if log_written {
        let _ = write!(summary, "Details: {log_path}");
    }
    summary
}
