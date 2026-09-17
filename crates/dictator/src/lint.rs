//! Lint command implementation

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::Source;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::cli::{LintArgs, OutputFormat};
use crate::config::{load_config, load_dictate_config};
use crate::dictate::{apply_fix_at_span, apply_single_fix};
use crate::diff;
use crate::files::{collect_all_files, detect_file_types, resolve_paths};
use crate::output::{
    SerializableDiagnostic, byte_to_line_col, print_diagnostic, print_github_annotation,
    print_persona_summary,
};
use crate::regime::init_regime_for_files;

pub fn run_once(
    args: LintArgs,
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    let cfg = load_config(config_path.as_ref())?;
    let format = match args.format.as_deref() {
        Some(raw) => raw.parse().map_err(anyhow::Error::msg)?,
        None => cfg.format.unwrap_or(OutputFormat::Human),
    };

    let selector = diff::selector_from_flags(args.diff.as_deref(), args.staged)?;
    let paths = resolve_paths(&args.paths, selector.is_some())?;
    let changed = selector
        .as_ref()
        .map(|sel| diff::changed_lines(sel, &paths, args.diff_context))
        .transpose()?;

    let mut files = collect_all_files(&paths)?;
    if let Some(changed) = &changed {
        files.retain(|f| changed.contains_file(f));
        if files.is_empty() {
            eprintln!("No changed files in range");
            return Ok(());
        }
    }
    if files.is_empty() {
        eprintln!("No files found");
        return Ok(());
    }

    let file_types = detect_file_types(&files);

    let decree_config = load_dictate_config(config_path.as_ref(), profile.as_deref())?;

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

    let personas = regime.personas();
    let scope = changed.map(|c| diff::Scope::new(c, regime.file_scope_rules()));

    let exit_code = Arc::new(Mutex::new(0));
    let json_out = Arc::new(Mutex::new(Vec::new()));
    let fixed_count = Arc::new(Mutex::new(0usize));
    let withheld = Arc::new(Mutex::new(0usize));

    // Process files in parallel using rayon
    files.par_iter().try_for_each(|path| -> Result<()> {
        let Some(text) = crate::files::read_source_file(path.as_std_path()) else {
            return Ok(());
        };
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
            let mut unique_diags: Vec<_> = diags
                .iter()
                .filter(|diag| {
                    seen.insert((
                        path_ref.as_str().to_string(),
                        diag.span.start,
                        diag.span.end,
                        diag.rule.clone(),
                        diag.message.clone(),
                        diag.enforced,
                    ))
                })
                .collect();

            if let Some(scope) = &scope {
                let before = unique_diags.len();
                unique_diags.retain(|diag| scope.allows(path_ref, &text, diag));
                *withheld.lock().unwrap() += before - unique_diags.len();
                if unique_diags.is_empty() {
                    return Ok(());
                }
            }

            if matches!(format, OutputFormat::Human) {
                print_persona_summary(path_ref.as_str(), &unique_diags, &personas);
            }

            for diag in unique_diags {
                // Apply fix if --fix is set and this is a fixable violation
                // Whole-file rewrite only for file-scope rules; line rules get
                // spliced so a scoped run leaves untouched lines alone.
                if args.fix
                    && diag.enforced
                    && let Some(new_text) = match &scope {
                        Some(s) if !s.is_file_scope(&diag.rule) => {
                            apply_fix_at_span(&fixed_text, diag)
                        }
                        _ => apply_single_fix(&fixed_text, diag),
                    }
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
                    OutputFormat::Github => {
                        let stdout = std::io::stdout();
                        let _handle = stdout.lock();
                        print_github_annotation(path_ref.as_str(), &text, diag);
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
    let total_withheld = *withheld.lock().unwrap();

    if args.fix && total_fixed > 0 {
        eprintln!("Fixed {total_fixed} file(s)");
    }

    if total_withheld > 0 {
        eprintln!("{total_withheld} legacy violation(s) withheld (outside the diff)");
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
