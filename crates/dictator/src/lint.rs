//! Lint command implementation

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::Source;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::cli::{LintArgs, OutputFormat};
use crate::config::load_config;
use crate::dictate::apply_single_fix;
use crate::files::{collect_all_files, detect_file_types};
use crate::output::{SerializableDiagnostic, byte_to_line_col, print_diagnostic};
use crate::regime::init_regime_for_files;

pub fn run_once(
    args: LintArgs,
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
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
    let mut decree_config = if let Some(p) = config_path.as_ref() {
        Some(dictator_core::DictateConfig::from_file(p.as_std_path())?)
    } else {
        dictator_core::DictateConfig::load_default_strict()?
    };

    // Apply profile if specified
    if let Some(ref config) = decree_config
        && let Some(ref profile_name) = profile
    {
        decree_config = Some(
            config
                .get_profile_config(profile_name)
                .map_err(|e| anyhow::anyhow!("Profile error: {e}"))?,
        );
    }

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
    #[cfg(feature = "wasm-loader")]
    for p in &args.plugin {
        regime.add_wasm_decree(p)?;
    }

    let exit_code = Arc::new(Mutex::new(0));
    let json_out = Arc::new(Mutex::new(Vec::new()));
    let fixed_count = Arc::new(Mutex::new(0usize));

    // Process files in parallel using rayon
    files.par_iter().try_for_each(|path| -> Result<()> {
        let text = fs::read_to_string(path)?;
        let path_ref = path.as_path();
        let source = Source {
            path: path_ref,
            text: &text,
        };
        let diags = regime.enforce(&[source])?;

        if !diags.is_empty() {
            let mut seen = HashSet::new();
            let mut fixed_text = text.clone();
            let mut file_was_fixed = false;

            for diag in &diags {
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

                // Apply fix if --fix is set and this is a fixable violation
                if args.fix
                    && diag.enforced
                    && let Some(new_text) = apply_single_fix(&fixed_text, diag)
                    && new_text != fixed_text
                {
                    fixed_text = new_text;
                    file_was_fixed = true;
                }

                match format {
                    OutputFormat::Human => {
                        let stdout = std::io::stdout();
                        let _handle = stdout.lock();
                        print_diagnostic(path_ref.as_str(), &text, diag);
                    }
                    OutputFormat::Json => {
                        let (line, col) = byte_to_line_col(&text, diag.span.start);
                        let mut json_guard = json_out.lock().unwrap();
                        json_guard.push(SerializableDiagnostic {
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
                *exit_code.lock().unwrap() = 1;
            }

            // Write fixed content if any fixes were applied
            if file_was_fixed {
                fs::write(path, &fixed_text)?;
                *fixed_count.lock().unwrap() += 1;
            }
        }
        Ok(())
    })?;

    let final_exit_code = *exit_code.lock().unwrap();
    let total_fixed = *fixed_count.lock().unwrap();

    if args.fix && total_fixed > 0 {
        eprintln!("Fixed {total_fixed} file(s)");
    }

    if matches!(format, OutputFormat::Json) {
        let output = {
            let json_guard = json_out.lock().unwrap();
            serde_json::to_string_pretty(&*json_guard)?
        };
        println!("{output}");
    }

    std::process::exit(final_exit_code);
}
