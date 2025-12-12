//! Lint command implementation

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::Source;
use std::collections::HashSet;
use std::fs;

use crate::cli::{LintArgs, OutputFormat};
use crate::config::load_config;
use crate::files::{collect_all_files, detect_file_types};
use crate::output::{SerializableDiagnostic, byte_to_line_col, print_diagnostic};
use crate::regime::init_regime_for_files;

pub fn run_once(args: LintArgs, config_path: Option<Utf8PathBuf>) -> Result<()> {
    let cfg = load_config(config_path.as_ref())?;
    let format = if args.json {
        OutputFormat::Json
    } else {
        cfg.format.unwrap_or(OutputFormat::Human)
    };

    let files = collect_all_files(&args.paths)?;
    if files.is_empty() {
        eprintln!("No files found");
        return Ok(());
    }

    let file_types = detect_file_types(&files);

    // Load decree configuration (with validation)
    let decree_config = if let Some(p) = config_path.as_ref() {
        Some(dictator_core::DictateConfig::from_file(p.as_std_path())?)
    } else {
        dictator_core::DictateConfig::load_default_strict()?
    };

    // Load native decrees based on detected file types
    let mut regime = init_regime_for_files(&file_types, decree_config.as_ref());

    // Load custom WASM decrees from config
    if let Some(ref config) = decree_config {
        for (name, settings) in &config.decree {
            // Skip built-in native decrees
            if matches!(
                name.as_str(),
                "supreme" | "ruby" | "typescript" | "golang" | "rust" | "python" | "frontmatter"
            ) {
                continue;
            }

            // Load custom decree if path provided and enabled
            if let Some(ref path) = settings.path
                && settings.enabled.unwrap_or(true)
            {
                regime.add_wasm_decree(path)?;
            }
        }
    }

    // Load any additional decrees from CLI
    for p in &args.plugin {
        regime.add_wasm_decree(p)?;
    }

    let mut exit_code = 0;
    let mut json_out: Vec<SerializableDiagnostic> = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path)?;
        let path_ref = path.as_path();
        let source = Source {
            path: path_ref,
            text: &text,
        };
        let diags = regime.enforce(&[source])?;
        let mut seen = HashSet::new();
        for diag in diags {
            let key = (
                path_ref.as_str().to_string(),
                diag.span.start,
                diag.span.end,
                diag.rule.clone(),
                diag.message.clone(),
                diag.enforced,
            );
            if !seen.insert(key) {
                continue;
            }
            match format {
                OutputFormat::Human => {
                    print_diagnostic(path_ref.as_str(), &text, &diag);
                }
                OutputFormat::Json => {
                    let (line, col) = byte_to_line_col(&text, diag.span.start);
                    json_out.push(SerializableDiagnostic {
                        path: path_ref.as_str().to_string(),
                        line,
                        col,
                        rule: diag.rule.clone(),
                        message: diag.message.clone(),
                        enforced: diag.enforced,
                        span: diag.span,
                    });
                }
            }
            exit_code = 1;
        }
    }

    if matches!(format, OutputFormat::Json) {
        println!("{}", serde_json::to_string_pretty(&json_out)?);
    }

    std::process::exit(exit_code);
}
