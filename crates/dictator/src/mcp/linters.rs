//! External linter execution and stalint checking.

use super::utils::make_snippet;
use camino::Utf8Path;
use dictator_core::{Regime, Source};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};

use super::protocol::{JsonRpcError, JsonRpcResponse};
use super::state::ServerState;
use super::utils::collect_files;

/// Run stalint check and return violations
pub fn run_stalint_check(paths: &[String]) -> Vec<Value> {
    let mut regime = Regime::new();

    let config = dictator_core::DictateConfig::load_default();
    regime.set_rule_ignores_from_config(config.as_ref());

    // Load decree configuration and apply to supreme plugin
    // Language-specific settings override supreme settings per file type
    if let Some(config) = config
        && let Some(supreme_settings) = config.decree.get("supreme")
    {
        let supreme_config = dictator_supreme::config_from_decree_settings(supreme_settings);

        // Build language overrides: merge supreme + language settings
        let mut overrides = std::collections::HashMap::new();
        for lang in ["ruby", "typescript", "golang", "rust", "python"] {
            if let Some(lang_settings) = config.decree.get(lang) {
                let merged = dictator_supreme::merged_config(supreme_settings, lang_settings);
                overrides.insert(lang.to_string(), merged);
            }
        }

        regime.add_decree(dictator_supreme::init_decree_with_overrides(
            supreme_config,
            overrides,
        ));

        // Load native decrees declared in config, applying per-decree settings
        for (decree_name, settings) in &config.decree {
            match decree_name.as_str() {
                "typescript" => {
                    let ts_config = dictator_typescript::config_from_decree_settings(settings);
                    let ts_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_typescript::init_decree_with_configs(
                        ts_config, ts_supreme,
                    ));
                }
                "python" => {
                    let py_config = dictator_python::config_from_decree_settings(settings);
                    let py_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_python::init_decree_with_configs(
                        py_config, py_supreme,
                    ));
                }
                "golang" => {
                    let go_config = dictator_golang::config_from_decree_settings(settings);
                    let go_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_golang::init_decree_with_configs(
                        go_config, go_supreme,
                    ));
                }
                "rust" => {
                    let rs_config = dictator_rust::config_from_decree_settings(settings);
                    let rs_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_rust::init_decree_with_configs(
                        rs_config, rs_supreme,
                    ));
                }
                "ruby" => {
                    let rb_config = dictator_ruby::config_from_decree_settings(settings);
                    let rb_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_ruby::init_decree_with_configs(
                        rb_config, rb_supreme,
                    ));
                }
                "frontmatter" => {
                    let fm_config = dictator_frontmatter::config_from_decree_settings(settings);
                    regime.add_decree(dictator_frontmatter::init_decree_with_config(fm_config));
                }
                _ => {} // Already loaded above; custom WASM decrees handled elsewhere
            }
        }
    } else {
        regime.add_decree(dictator_supreme::init_decree());
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    let mut violations = Vec::new();

    for path in paths {
        let path = std::path::Path::new(path);
        if !path.exists() {
            continue;
        }

        let files = collect_files(path);
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };

            // Use relative path if within cwd (saves tokens)
            let relative = file.strip_prefix(&cwd).unwrap_or(&file);
            let path_str = relative.to_str().unwrap_or("<invalid>");
            let source = Source {
                path: Utf8Path::new(path_str),
                text: &text,
            };

            if let Ok(diags) = regime.enforce(&[source]) {
                for diag in &diags {
                    let snippet = make_snippet(&text, &diag.span, 160);
                    violations.push(serde_json::json!({
                        "file": path_str,
                        "rule": diag.rule,
                        "message": diag.message,
                        "enforced": diag.enforced,
                        "snippet": snippet,
                    }));
                }
            }
        }
    }

    violations
}

/// Get linter args based on command - Dictator controls the format for parsing
pub fn get_linter_args(command: &str) -> Vec<&'static str> {
    match command {
        "rubocop" => vec!["-A", "--format", "json"],
        "eslint" => vec!["--fix", "--format", "json"],
        "ruff" => vec!["check", "--fix", "--output-format", "json"],
        "prettier" => vec!["--write"],
        "gofmt" | "goimports" => vec!["-w"],
        "rustfmt" => vec!["--edition", "2021"],
        "clippy" | "cargo-clippy" => vec!["--fix", "--allow-dirty", "--message-format", "json"],
        _ => vec![], // black and unknown linters run without args
    }
}

/// Handle kimjongrails auto-fix (whitespace, newlines, line endings)
pub fn handle_kimjongrails(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    use serde::Deserialize;
    use std::collections::HashMap;

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

        let mut fixed = text.clone();
        let mut changes = Vec::new();

        // Fix trailing whitespace
        let lines: Vec<&str> = fixed.lines().collect();
        let trimmed: Vec<String> = lines.iter().map(|l| l.trim_end().to_string()).collect();
        if lines.iter().zip(trimmed.iter()).any(|(a, b)| *a != b) {
            fixed = trimmed.join("\n");
            changes.push("trailing whitespace");
        }

        // Ensure final newline
        if !fixed.ends_with('\n') {
            fixed.push('\n');
            changes.push("final newline");
        }

        // Normalize line endings to LF
        if fixed.contains("\r\n") {
            fixed = fixed.replace("\r\n", "\n");
            changes.push("CRLF->LF");
        }

        if !changes.is_empty() && fixed != text {
            if let Err(e) = std::fs::write(file, &fixed) {
                let _ = writeln!(log_output, "! Cannot write {}: {}", file.display(), e);
            } else {
                fixed_count += 1;
                let _ = writeln!(log_output, "* {} ({})", file.display(), changes.join(", "));

                // Track counts by rule
                for change in &changes {
                    let rule = match *change {
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

    let output = if fixed_count == 0 {
        "No fixes needed".to_string()
    } else {
        // Write log file with details
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let log_path = format!("/tmp/dictator-fixes-{timestamp}.log");

        let log_written = std::fs::write(&log_path, &log_output).is_ok();

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
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [{ "type": "text", "text": output }]
        })),
        error: None,
    }
}

/// Handle supremecourt - runs configured external linters
pub fn handle_supremecourt(
    id: Value,
    arguments: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    use serde::Deserialize;

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
                "content": [{ "type": "text", "text": "No .dictate.toml found - no linters configured" }]
            })),
            error: None,
        };
    }
    let config = config.unwrap();

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

    // Detect which decrees have files
    let mut decrees_with_files: HashSet<String> = HashSet::new();
    for path in &args.paths {
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

    let mut output = String::new();
    let paths_str: Vec<&str> = args.paths.iter().map(std::string::String::as_str).collect();

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
            // Dictator controls the args based on linter type
            let args = get_linter_args(&linter.command);
            let _ = writeln!(
                output,
                "\n>> Dictator conducting {} inquisition...",
                linter.command
            );

            // For linters without JSON output (gofmt, rustfmt, etc.), list files first
            let has_json_output = matches!(
                linter.command.as_str(),
                "rubocop" | "eslint" | "ruff" | "clippy" | "cargo-clippy"
            );

            if has_json_output {
                // Linters with JSON output
                match Command::new(&linter.command)
                    .args(&args)
                    .args(&paths_str)
                    .output()
                {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        // Parse JSON output to unified Diagnostic format
                        let diagnostics = dictator_core::linter_output::parse_linter_output(
                            &linter.command,
                            &stdout,
                        );

                        if diagnostics.is_empty() {
                            output.push_str("* No issues found\n");
                        } else {
                            for diag in diagnostics {
                                let prefix = if diag.enforced { ">>" } else { "!!" };
                                let _ =
                                    writeln!(output, "{} {}: {}", prefix, diag.rule, diag.message);
                            }
                        }

                        // Also show stderr if any
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        if !stderr.trim().is_empty() {
                            let _ = writeln!(output, "stderr: {stderr}");
                        }
                    }
                    Err(e) => {
                        let _ = writeln!(output, "! {} failed: {}", linter.command, e);
                    }
                }
            } else {
                // Run with -l first to see what files need fixing (gofmt/goimports)
                if matches!(linter.command.as_str(), "gofmt" | "goimports") {
                    if let Ok(list_output) = Command::new(&linter.command)
                        .arg("-l")
                        .args(&paths_str)
                        .output()
                    {
                        let files_to_fix = String::from_utf8_lossy(&list_output.stdout);
                        let files: Vec<&str> =
                            files_to_fix.lines().filter(|l| !l.is_empty()).collect();
                        if files.is_empty() {
                            output.push_str("* No issues found\n");
                        } else {
                            for file in &files {
                                let _ = writeln!(output, ">> {file}");
                            }
                            // Now actually fix them
                            let _ = Command::new(&linter.command)
                                .args(&args)
                                .args(&paths_str)
                                .output();
                        }
                    }
                } else {
                    // Other formatters (rustfmt, black, prettier) - just run
                    match Command::new(&linter.command)
                        .args(&args)
                        .args(&paths_str)
                        .output()
                    {
                        Ok(_) => output.push_str("* Formatting applied\n"),
                        Err(e) => {
                            let _ = writeln!(output, "! {} failed: {}", linter.command, e);
                        }
                    }
                }
            }
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

/// Initialize regime with configured decrees
pub fn init_regime_from_config() -> Regime {
    let mut regime = Regime::new();

    let config = dictator_core::DictateConfig::load_default();
    regime.set_rule_ignores_from_config(config.as_ref());

    // Load decree configuration and apply to supreme plugin
    // Language-specific settings override supreme settings per file type
    if let Some(config) = config
        && let Some(supreme_settings) = config.decree.get("supreme")
    {
        let supreme_config = dictator_supreme::config_from_decree_settings(supreme_settings);

        // Build language overrides: merge supreme + language settings
        let mut overrides = std::collections::HashMap::new();
        for lang in ["ruby", "typescript", "golang", "rust", "python"] {
            if let Some(lang_settings) = config.decree.get(lang) {
                let merged = dictator_supreme::merged_config(supreme_settings, lang_settings);
                overrides.insert(lang.to_string(), merged);
            }
        }

        regime.add_decree(dictator_supreme::init_decree_with_overrides(
            supreme_config,
            overrides,
        ));

        // Load native decrees declared in config with overrides
        for (decree_name, settings) in &config.decree {
            match decree_name.as_str() {
                "typescript" => {
                    let ts_config = dictator_typescript::config_from_decree_settings(settings);
                    let ts_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_typescript::init_decree_with_configs(
                        ts_config, ts_supreme,
                    ));
                }
                "python" => {
                    let py_config = dictator_python::config_from_decree_settings(settings);
                    let py_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_python::init_decree_with_configs(
                        py_config, py_supreme,
                    ));
                }
                "golang" => {
                    let go_config = dictator_golang::config_from_decree_settings(settings);
                    let go_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_golang::init_decree_with_configs(
                        go_config, go_supreme,
                    ));
                }
                "rust" => {
                    let rs_config = dictator_rust::config_from_decree_settings(settings);
                    let rs_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_rust::init_decree_with_configs(
                        rs_config, rs_supreme,
                    ));
                }
                "ruby" => {
                    let rb_config = dictator_ruby::config_from_decree_settings(settings);
                    let rb_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                    regime.add_decree(dictator_ruby::init_decree_with_configs(
                        rb_config, rb_supreme,
                    ));
                }
                "frontmatter" => {
                    let fm_config = dictator_frontmatter::config_from_decree_settings(settings);
                    regime.add_decree(dictator_frontmatter::init_decree_with_config(fm_config));
                }
                _ => {} // Already loaded above; custom WASM decrees handled elsewhere
            }
        }
    } else {
        regime.add_decree(dictator_supreme::init_decree());
    }

    regime
}
