//! Dictate command implementation - structural issue fixing

use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use dictator_core::DictateConfig;
use dictator_decree_abi::Diagnostic;
use std::fs;

use crate::cli::DictateArgs;
use crate::config::load_dictate_config;
use crate::diff::{self, ChangedLines};
use crate::files::{collect_all_files, resolve_paths};
use crate::interactive::InteractiveFixer;
use crate::regime::init_regime_for_files;

pub fn run_dictate(
    args: DictateArgs,
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    let selector = diff::selector_from_flags(args.diff.as_deref(), args.staged)?;
    let paths = resolve_paths(&args.paths, selector.is_some())?;
    let changed = selector
        .as_ref()
        .map(|sel| diff::changed_lines(sel, &paths, args.diff_context))
        .transpose()?;

    if args.interactive {
        run_interactive_dictate(&paths, changed, config_path, profile)
    } else {
        run_batch_dictate(&paths, changed, config_path, profile)
    }
}

/// Run interactive fix mode
fn run_interactive_dictate(
    paths: &[Utf8PathBuf],
    changed: Option<ChangedLines>,
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    println!("🔧 Interactive Fix Mode");
    println!("========================\n");

    let decree_config = load_dictate_config(config_path.as_ref(), profile.as_deref())?;

    let mut fixer = InteractiveFixer::new();

    println!("🔍 Collecting fixable violations...");
    fixer.collect_violations(paths, changed.as_ref(), decree_config.as_ref())?;
    fixer.run_interactive(false)?;

    Ok(())
}

/// Run batch fix mode (original behavior)
fn run_batch_dictate(
    paths: &[Utf8PathBuf],
    changed: Option<ChangedLines>,
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    let mut files = collect_all_files(paths)?;
    if let Some(changed) = &changed {
        files.retain(|f| changed.contains_file(f));
    }
    if files.is_empty() {
        eprintln!("No files found");
        return Ok(());
    }

    let decree_config = load_dictate_config(config_path.as_ref(), profile.as_deref())?;

    let file_types = crate::files::detect_file_types(&files);
    let mut regime = init_regime_for_files(&file_types, decree_config.as_ref());

    // Load custom WASM decrees from config
    if let Some(ref config) = decree_config {
        for (name, settings) in &config.decree {
            if matches!(
                name.as_str(),
                "supreme" | "ruby" | "typescript" | "golang" | "rust" | "python" | "frontmatter"
            ) {
                continue;
            }

            if let Some(ref path) = settings.path
                && settings.enabled.unwrap_or(true)
            {
                regime.add_wasm_decree(path)?;
            }
        }
    }

    let scope = changed.map(|c| diff::Scope::new(c, regime.file_scope_rules()));

    let mut fixed_count = 0;
    let mut file_count = 0;

    for path in files {
        let Some(original) = crate::files::read_source_file(path.as_std_path()) else {
            continue;
        };
        let source = dictator_core::Source {
            path: path.as_path(),
            text: &original,
        };

        let diags = regime.enforce(&[source])?;
        let (fixed_text, rules) = fix_source(
            path.as_path(),
            &original,
            &diags,
            scope.as_ref(),
            decree_config.as_ref(),
        );

        if !rules.is_empty() {
            fs::write(&path, &fixed_text)?;
            println!("Fixed: {path}");
            fixed_count += 1;
        }
        file_count += 1;
    }

    if fixed_count > 0 {
        println!("Fixed {fixed_count} file(s) out of {file_count} checked.");
    } else {
        println!("All {file_count} file(s) already compliant.");
    }

    Ok(())
}

/// Apply every enforced fix in `diags` to `original`, in order.
///
/// Returns the fixed text and the rule of each fix that changed it. With a
/// `scope`, diagnostics outside it are skipped and line rules are confined to
/// their own lines.
pub(crate) fn fix_source<'a>(
    path: &Utf8Path,
    original: &str,
    diags: impl IntoIterator<Item = &'a Diagnostic>,
    scope: Option<&diff::Scope>,
    config: Option<&DictateConfig>,
) -> (String, Vec<String>) {
    let mut fixed_text = original.to_string();
    let mut rules = Vec::new();

    for diag in diags {
        // Spans index the original text, so scope-check before any rewrite.
        if scope.is_some_and(|s| !s.allows(path, original, diag)) {
            continue;
        }
        // Whole-file rewrite only for file-scope rules; line rules get
        // spliced so a scoped run leaves untouched lines alone.
        if diag.enforced
            && let Some(new_text) = match scope {
                Some(s) if !s.is_file_scope(&diag.rule) => {
                    apply_fix_at_span(&fixed_text, diag, config)
                }
                _ => apply_single_fix(&fixed_text, diag, config),
            }
            && new_text != fixed_text
        {
            fixed_text = new_text;
            rules.push(diag.rule.clone());
        }
    }

    (fixed_text, rules)
}

/// Apply `diag`'s fix but confine the rewrite to the lines its span covers.
///
/// The fixers below rewrite the whole file. Splicing the untouched lines back is
/// what stops `--diff --fix` from reformatting code the author never edited.
/// Only valid for line-anchored rules; file-scope rules must fix whole-file.
pub(crate) fn apply_fix_at_span(
    content: &str,
    diag: &Diagnostic,
    config: Option<&DictateConfig>,
) -> Option<String> {
    let fixed = apply_single_fix(content, diag, config)?;

    let (start, _) = crate::output::byte_to_line_col(content, diag.span.start);
    let (end, _) = crate::output::byte_to_line_col(content, diag.span.end.max(diag.span.start));

    let original_lines: Vec<&str> = content.split('\n').collect();
    let fixed_lines: Vec<&str> = fixed.split('\n').collect();
    if original_lines.len() != fixed_lines.len() {
        return Some(fixed);
    }

    let spliced: Vec<&str> = original_lines
        .iter()
        .enumerate()
        .map(|(idx, original)| {
            if (start..=end).contains(&(idx + 1)) {
                fixed_lines[idx]
            } else {
                *original
            }
        })
        .collect();
    Some(spliced.join("\n"))
}

/// Apply a single fix based on a diagnostic
pub(crate) fn apply_single_fix(
    content: &str,
    diag: &Diagnostic,
    config: Option<&DictateConfig>,
) -> Option<String> {
    match diag.rule.as_str() {
        rule if rule.contains("trailing-whitespace") => Some(map_lines(content, |line| {
            line.trim_end_matches([' ', '\t'])
        })),
        rule if rule.contains("tab-character") => Some(content.replace('\t', "  ")),
        rule if rule.contains("missing-final-newline") => add_final_newline(content),
        rule if rule.contains("mixed-line-endings") || rule.contains("wrong-line-ending") => {
            Some(normalize_line_endings(content, wants_crlf(config, rule)))
        }
        rule if rule.contains("blank-line-whitespace") => Some(map_lines(content, |line| {
            // Keep content unless it's a line with only whitespace
            if line.trim().is_empty() { "" } else { line }
        })),
        "ruby/comment-space" => Some(dictator_ruby::fix_comment_spacing(content)),
        _ => None, // Not a fixable rule we handle
    }
}

/// Append a final newline in the file's own line ending; `None` if present.
pub(crate) fn add_final_newline(content: &str) -> Option<String> {
    if content.ends_with('\n') {
        None
    } else if content.contains("\r\n") {
        Some(format!("{content}\r\n"))
    } else {
        Some(format!("{content}\n"))
    }
}

/// Convert every line ending to CRLF or LF.
pub(crate) fn normalize_line_endings(content: &str, crlf: bool) -> String {
    let lf = content.replace("\r\n", "\n");
    if crlf { lf.replace('\n', "\r\n") } else { lf }
}

/// Whether `rule`'s owner enforces CRLF: the owning decree's `line_endings`,
/// else `decree.supreme`'s, LF when neither says. Mirrors how decrees merge
/// supreme settings under a language override.
pub(crate) fn wants_crlf(config: Option<&DictateConfig>, rule: &str) -> bool {
    let setting = |decree: &str| config?.decree.get(decree)?.line_endings.as_deref();
    let owner = rule.split_once('/').map_or("supreme", |(owner, _)| owner);
    setting(owner).or_else(|| setting("supreme")) == Some("crlf")
}

/// Rewrite each line's content with `f`, keeping every line terminator as is.
fn map_lines<'a>(content: &'a str, f: impl Fn(&'a str) -> &'a str) -> String {
    content
        .split_inclusive('\n')
        .flat_map(|chunk| {
            let body = chunk.strip_suffix('\n').unwrap_or(chunk);
            let body = body.strip_suffix('\r').unwrap_or(body);
            [f(body), &chunk[body.len()..]]
        })
        .collect()
}

/// Fix structural issues: trailing whitespace, line endings, final newline, blank line whitespace
#[cfg(test)]
fn fix_structural_issues(content: &str) -> String {
    let mut result = String::with_capacity(content.len());

    // Normalize CRLF to LF
    let normalized = content.replace("\r\n", "\n");

    for line in normalized.split('\n') {
        // Remove trailing whitespace from each line
        let trimmed = line.trim_end_matches([' ', '\t']);
        result.push_str(trimmed);
        result.push('\n');
    }

    // Remove the extra newline we added after the last line
    if result.ends_with('\n') && !normalized.ends_with('\n') {
        // Original didn't end with newline, but we want to add one
        // So keep the newline we added
    } else if result.ends_with("\n\n") && !normalized.ends_with("\n\n") {
        // We added an extra newline, remove it
        result.pop();
    }

    // Ensure exactly one final newline
    while result.ends_with("\n\n") {
        result.pop();
    }
    if !result.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests;
