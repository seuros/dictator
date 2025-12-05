//! External linter execution and stalint checking.

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

    // Load decree configuration and apply to supreme plugin
    // Language-specific settings override supreme settings per file type
    if let Some(config) = dictator_core::DictateConfig::load_default()
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
                    regime.add_decree(dictator_typescript::init_decree_with_config(ts_config));
                }
                "python" => {
                    let py_config = dictator_python::config_from_decree_settings(settings);
                    regime.add_decree(dictator_python::init_decree_with_config(py_config));
                }
                "golang" => {
                    let go_config = dictator_golang::config_from_decree_settings(settings);
                    regime.add_decree(dictator_golang::init_decree_with_config(go_config));
                }
                "rust" => {
                    let rs_config = dictator_rust::config_from_decree_settings(settings);
                    regime.add_decree(dictator_rust::init_decree_with_config(rs_config));
                }
                "ruby" => {
                    let rb_config = dictator_ruby::config_from_decree_settings(settings);
                    regime.add_decree(dictator_ruby::init_decree_with_config(rb_config));
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
                    violations.push(serde_json::json!({
                        "file": path_str,
                        "rule": diag.rule,
                        "message": diag.message,
                        "enforced": diag.enforced
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
pub fn handle_kimjongrails(id: Value, arguments: Option<Value>) -> JsonRpcResponse {
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

    let mut output = String::new();
    let mut fixed_count = 0;

    for path in &args.paths {
        let path = std::path::Path::new(path);
        if !path.exists() {
            let _ = writeln!(output, "! Path not found: {}", path.display());
            continue;
        }

        let files = collect_files(path);
        for file in files {
            let text = match std::fs::read_to_string(&file) {
                Ok(t) => t,
                Err(e) => {
                    let _ = writeln!(output, "! Cannot read {}: {}", file.display(), e);
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
                if let Err(e) = std::fs::write(&file, &fixed) {
                    let _ = writeln!(output, "! Cannot write {}: {}", file.display(), e);
                } else {
                    fixed_count += 1;
                    let _ = writeln!(output, "* {} ({})", file.display(), changes.join(", "));
                }
            }
        }
    }

    if fixed_count == 0 {
        output = "* No fixes needed".to_string();
    } else {
        output = format!("Fixed {fixed_count} file(s):\n{output}");
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

    // Load config to get linter configurations
    let state = watcher_state.lock().unwrap();
    let config = state.config.as_ref();

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

    // Run configured linters for each detected decree
    for decree_name in &decrees_with_files {
        if let Some(decree) = config.unwrap().decree.get(decree_name)
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

    drop(state);

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

    // Load decree configuration and apply to supreme plugin
    // Language-specific settings override supreme settings per file type
    if let Some(config) = dictator_core::DictateConfig::load_default()
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
                    regime.add_decree(dictator_typescript::init_decree_with_config(ts_config));
                }
                "python" => {
                    let py_config = dictator_python::config_from_decree_settings(settings);
                    regime.add_decree(dictator_python::init_decree_with_config(py_config));
                }
                "golang" => {
                    let go_config = dictator_golang::config_from_decree_settings(settings);
                    regime.add_decree(dictator_golang::init_decree_with_config(go_config));
                }
                "rust" => {
                    let rs_config = dictator_rust::config_from_decree_settings(settings);
                    regime.add_decree(dictator_rust::init_decree_with_config(rs_config));
                }
                "ruby" => {
                    let rb_config = dictator_ruby::config_from_decree_settings(settings);
                    regime.add_decree(dictator_ruby::init_decree_with_config(rb_config));
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
