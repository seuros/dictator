//! External linter execution for MCP tools.

use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};

use super::protocol::{JsonRpcError, JsonRpcResponse};
use super::state::ServerState;
use super::utils::collect_files;

/// Get linter args based on command - Dictator controls the format for parsing
pub fn get_linter_args(command: &str) -> Vec<&'static str> {
    match command {
        "rubocop" => vec!["-A", "--format", "json"],
        "eslint" => vec!["--fix", "--format", "json"],
        "ruff" => vec!["check", "--fix", "--output-format", "json"],
        "prettier" => vec!["--write"],
        "gofmt" | "goimports" => vec!["-w"],
        "rustfmt" => vec!["--edition", "2021"],
        "clippy" | "cargo-clippy" => {
            vec!["--fix", "--allow-dirty", "--message-format", "json"]
        }
        _ => vec![], // black and unknown linters run without args
    }
}

/// Handle supremecourt - runs configured external linters
pub fn handle_supremecourt(
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

    // Load config to get linter configurations (clone to release lock early)
    let config = {
        let state = watcher_state.lock().unwrap();
        state.config.clone()
    };

    if config.is_none() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": "No .dictate.toml found - no linters configured"
                }]
            })),
            error: None,
        };
    }
    let config = config.unwrap();

    let decrees_with_files = detect_decrees_with_files(&args.paths);

    let mut output = String::new();
    let paths_str: Vec<&str> = args.paths.iter().map(String::as_str).collect();

    // Start progress tracking
    let progress_token = {
        let state = watcher_state.lock().unwrap();
        let total = u32::try_from(decrees_with_files.len()).unwrap_or(u32::MAX);
        state.progress_tracker.start("supremecourt", total)
    };

    // Run configured linters for each detected decree
    for (decree_idx, decree_name) in decrees_with_files.iter().enumerate() {
        // Update progress
        {
            let state = watcher_state.lock().unwrap();
            let current = u32::try_from(decree_idx + 1).unwrap_or(u32::MAX);
            state.progress_tracker.progress(&progress_token, current);
        }

        if let Some(decree) = config.decree.get(decree_name)
            && let Some(linter) = &decree.linter
        {
            run_linter(&linter.command, &paths_str, &mut output);
        }
    }

    // Finish progress tracking
    {
        let state = watcher_state.lock().unwrap();
        state.progress_tracker.finish(&progress_token);
    }

    if output.is_empty() {
        output = "No configured linters for detected file types".to_string();
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": output }]
        })),
        error: None,
    }
}

/// Detect which decrees have files in the given paths
fn detect_decrees_with_files(paths: &[String]) -> HashSet<String> {
    // Map file extensions to decree names
    let ext_to_decree: HashMap<&str, &str> = HashMap::from([
        ("rb", "ruby"),
        ("rake", "ruby"),
        ("js", "typescript"),
        ("jsx", "typescript"),
        ("ts", "typescript"),
        ("tsx", "typescript"),
        ("py", "python"),
        ("go", "golang"),
        ("rs", "rust"),
    ]);

    let mut decrees_with_files: HashSet<String> = HashSet::new();
    for path in paths {
        let path = std::path::Path::new(path);
        let files = collect_files(path);
        for file in files {
            if let Some(ext) = file.extension().and_then(|e| e.to_str())
                && let Some(decree_name) = ext_to_decree.get(ext)
            {
                decrees_with_files.insert((*decree_name).to_string());
            }
        }
    }
    decrees_with_files
}

/// Run a single linter and append output
fn run_linter(command: &str, paths: &[&str], output: &mut String) {
    let args = get_linter_args(command);
    let _ = writeln!(output, "\n>> Dictator conducting {command} inquisition...");

    // For linters without JSON output (gofmt, rustfmt, etc.), list files first
    let has_json_output = matches!(
        command,
        "rubocop" | "eslint" | "ruff" | "clippy" | "cargo-clippy"
    );

    if has_json_output {
        run_json_linter(command, &args, paths, output);
    } else if matches!(command, "gofmt" | "goimports") {
        run_go_formatter(command, &args, paths, output);
    } else {
        run_basic_formatter(command, &args, paths, output);
    }
}

/// Run linter with JSON output
fn run_json_linter(command: &str, args: &[&str], paths: &[&str], output: &mut String) {
    match Command::new(command).args(args).args(paths).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let diagnostics = dictator_core::linter_output::parse_linter_output(command, &stdout);

            if diagnostics.is_empty() {
                output.push_str("* No issues found\n");
            } else {
                for diag in diagnostics {
                    let prefix = if diag.enforced { ">>" } else { "!!" };
                    let _ = writeln!(output, "{} {}: {}", prefix, diag.rule, diag.message);
                }
            }

            // Also show stderr if any
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !stderr.trim().is_empty() {
                let _ = writeln!(output, "stderr: {stderr}");
            }
        }
        Err(e) => {
            let _ = writeln!(output, "! {command} failed: {e}");
        }
    }
}

/// Run Go formatters (gofmt/goimports) which use -l to list files
fn run_go_formatter(command: &str, args: &[&str], paths: &[&str], output: &mut String) {
    // Run with -l first to see what files need fixing
    if let Ok(list_output) = Command::new(command).arg("-l").args(paths).output() {
        let files_to_fix = String::from_utf8_lossy(&list_output.stdout);
        let files: Vec<&str> = files_to_fix.lines().filter(|l| !l.is_empty()).collect();
        if files.is_empty() {
            output.push_str("* No issues found\n");
        } else {
            for file in &files {
                let _ = writeln!(output, ">> {file}");
            }
            // Now actually fix them
            let _ = Command::new(command).args(args).args(paths).output();
        }
    }
}

/// Run basic formatters (rustfmt, black, prettier)
fn run_basic_formatter(command: &str, args: &[&str], paths: &[&str], output: &mut String) {
    match Command::new(command).args(args).args(paths).output() {
        Ok(_) => output.push_str("* Formatting applied\n"),
        Err(e) => {
            let _ = writeln!(output, "! {command} failed: {e}");
        }
    }
}
