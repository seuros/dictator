//! Dictate command implementation - structural issue fixing

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::DictateConfig;
use std::fs;

use crate::cli::DictateArgs;
use crate::files::collect_all_files;
use crate::interactive::InteractiveFixer;
use crate::regime::init_regime_for_files;

pub fn run_dictate(
    args: DictateArgs,
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    if args.interactive {
        run_interactive_dictate(&args.paths, config_path, profile)
    } else {
        run_batch_dictate(&args.paths, config_path, profile)
    }
}

/// Run interactive fix mode
fn run_interactive_dictate(
    paths: &[Utf8PathBuf],
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    println!("🔧 Interactive Fix Mode");
    println!("========================\n");

    // Load configuration
    let mut decree_config = if let Some(p) = config_path.as_ref() {
        Some(DictateConfig::from_file(p.as_std_path())?)
    } else {
        DictateConfig::load_default_strict()?
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

    let mut fixer = InteractiveFixer::new();

    println!("🔍 Collecting fixable violations...");
    fixer.collect_violations(paths, decree_config.as_ref())?;

    if !fixer.has_violations() {
        println!("✨ No fixable violations found!");
        return Ok(());
    }

    println!("🔧 Found {} fixable violations\n", fixer.violation_count());

    fixer.run_interactive(false)?;

    Ok(())
}

/// Run batch fix mode (original behavior)
fn run_batch_dictate(
    paths: &[Utf8PathBuf],
    config_path: Option<Utf8PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    let files = collect_all_files(paths)?;
    if files.is_empty() {
        eprintln!("No files found");
        return Ok(());
    }

    // Load configuration for enhanced batch mode
    let mut decree_config = if let Some(p) = config_path.as_ref() {
        Some(DictateConfig::from_file(p.as_std_path())?)
    } else {
        DictateConfig::load_default_strict()?
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
        let mut fixed_text = original.clone();
        let mut file_was_fixed = false;

        // Apply all fixable violations
        for diag in diags {
            if diag.enforced
                && let Some(new_text) = apply_single_fix(&fixed_text, &diag)
                && new_text != fixed_text
            {
                fixed_text = new_text;
                file_was_fixed = true;
            }
        }

        if file_was_fixed {
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

/// Apply a single fix based on a diagnostic
pub(crate) fn apply_single_fix(
    content: &str,
    diag: &dictator_decree_abi::Diagnostic,
) -> Option<String> {
    match diag.rule.as_str() {
        rule if rule.contains("trailing-whitespace") => {
            let mut result = String::with_capacity(content.len());
            for line in content.lines() {
                result.push_str(line.trim_end_matches([' ', '\t']));
                result.push('\n');
            }
            // Remove final newline if original didn't have one
            if !content.ends_with('\n') && result.ends_with('\n') {
                result.pop();
            }
            Some(result)
        }
        rule if rule.contains("tab-character") => Some(content.replace('\t', "  ")),
        rule if rule.contains("missing-final-newline") => {
            if content.ends_with('\n') {
                None // Already has final newline
            } else {
                Some(format!("{content}\n"))
            }
        }
        rule if rule.contains("mixed-line-endings") => Some(content.replace("\r\n", "\n")),
        rule if rule.contains("blank-line-whitespace") => {
            let mut result = String::with_capacity(content.len());
            for line in content.lines() {
                // Keep content unless it's a line with only whitespace
                if !line.trim().is_empty() || line.is_empty() {
                    result.push_str(line);
                }
                result.push('\n');
            }
            // Handle final newline
            if !content.ends_with('\n') && result.ends_with('\n') {
                result.pop();
            }
            Some(result)
        }
        _ => None, // Not a fixable rule we handle
    }
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
mod tests {
    use super::*;

    #[test]
    fn test_fix_trailing_whitespace() {
        let input = "hello world   \nfoo bar\t\t\n";
        let expected = "hello world\nfoo bar\n";
        assert_eq!(fix_structural_issues(input), expected);
    }

    #[test]
    fn test_fix_crlf() {
        let input = "hello\r\nworld\r\n";
        let expected = "hello\nworld\n";
        assert_eq!(fix_structural_issues(input), expected);
    }

    #[test]
    fn test_add_final_newline() {
        let input = "hello world";
        let expected = "hello world\n";
        assert_eq!(fix_structural_issues(input), expected);
    }

    #[test]
    fn test_remove_extra_final_newlines() {
        let input = "hello world\n\n\n";
        let expected = "hello world\n";
        assert_eq!(fix_structural_issues(input), expected);
    }
}
