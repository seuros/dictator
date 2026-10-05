//! Auto-fix handlers for MCP tools.

use camino::Utf8Path;
use dictator_core::Source;
use mcp_host::protocol::types::JsonRpcResponse;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::regime::{init_regime, load_config};
use super::state::ServerState;
use super::utils::{collect_files, parse_arguments};
use crate::dictate::fix_source;

/// Handle kimjongrails auto-fix: every enforced fix the workspace's decrees
/// offer, the same fixes `dictator dictate` applies.
pub fn handle_kimjongrails(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct Args {
        paths: Vec<String>,
        #[serde(default)]
        workspace: Option<String>,
    }

    let args: Args = match parse_arguments(&id, arguments) {
        Ok(args) => args,
        Err(response) => return *response,
    };

    // Same config, ignores and decrees stalint judged the files by.
    let config = load_config(args.workspace.as_deref().map(Path::new));
    let regime = init_regime(config.as_ref());

    let mut log_output = String::new();
    let mut fixed_count = 0;
    let mut rule_counts: HashMap<String, usize> = HashMap::new();

    // Collect all files first for progress tracking
    let all_files: Vec<std::path::PathBuf> = args
        .paths
        .iter()
        .map(Path::new)
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

        let Some(path) = Utf8Path::from_path(file) else {
            let _ = writeln!(log_output, "! Skipping non-UTF-8 path {}", file.display());
            continue;
        };
        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => {
                let _ = writeln!(log_output, "! Cannot read {path}: {e}");
                continue;
            }
        };
        let diags = match regime.enforce(&[Source { path, text: &text }]) {
            Ok(diags) => diags,
            Err(e) => {
                let _ = writeln!(log_output, "! Cannot lint {path}: {e}");
                continue;
            }
        };

        let (fixed, rules) = fix_source(path, &text, &diags, None, config.as_ref());
        if rules.is_empty() {
            continue;
        }
        if let Err(e) = std::fs::write(file, &fixed) {
            let _ = writeln!(log_output, "! Cannot write {path}: {e}");
            continue;
        }
        fixed_count += 1;
        let _ = writeln!(log_output, "* {path} ({})", rules.join(", "));
        for rule in rules {
            *rule_counts.entry(rule).or_default() += 1;
        }
    }

    // Finish progress tracking
    {
        let state = watcher_state.lock().unwrap();
        state.progress_tracker.finish(&progress_token);
    }

    let output = build_summary(fixed_count, &rule_counts, &log_output);

    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: Some(id),
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": output }]
        })),
        error: None,
    }
}

/// Build summary output for fixes
fn build_summary(
    fixed_count: usize,
    rule_counts: &HashMap<String, usize>,
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

#[cfg(test)]
mod tests;
