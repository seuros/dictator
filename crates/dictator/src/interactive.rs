//! Interactive fix mode - allows users to review and selectively apply fixes

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::DictateConfig;
use std::fs;
use std::io::{self, Write};

/// A fixable violation. The fix is computed against the file as it is when
/// shown or applied, so fixes accepted earlier in the session survive.
#[derive(Debug, Clone)]
pub struct FixableViolation {
    pub path: Utf8PathBuf,
    pub line: usize,
    pub column: usize,
    pub rule: String,
    pub message: String,
}

impl FixableViolation {
    /// `text` with this fix applied plus a description, or `None` when the
    /// rule has no fixer here or the fix changes nothing.
    fn fix(&self, text: &str) -> Option<(String, &'static str)> {
        let (fixed, description) = match self.rule.as_str() {
            rule if rule.contains("trailing-whitespace") => (
                map_line(text, self.line, |l| {
                    l.trim_end_matches([' ', '\t']).to_string()
                })?,
                "Remove trailing whitespace",
            ),
            rule if rule.contains("tab-character") => (
                map_line(text, self.line, |l| l.replace('\t', "  "))?,
                "Replace tabs with spaces",
            ),
            "ruby/comment-space" => {
                // Whole-file pass: heredoc detection needs the context.
                let all = dictator_ruby::fix_comment_spacing(text);
                let fixed_line = all.split('\n').nth(self.line - 1)?;
                let fixed_line = fixed_line.strip_suffix('\r').unwrap_or(fixed_line);
                (
                    map_line(text, self.line, |_| fixed_line.to_string())?,
                    "Add a space after #",
                )
            }
            rule if rule.contains("missing-final-newline") && !text.ends_with('\n') => {
                (format!("{text}\n"), "Add final newline")
            }
            _ => return None,
        };
        (fixed != text).then_some((fixed, description))
    }
}

/// Rewrite 1-based `line` with `f`, keeping every line terminator intact.
fn map_line(text: &str, line: usize, f: impl FnOnce(&str) -> String) -> Option<String> {
    let start: usize = text
        .split_inclusive('\n')
        .take(line.checked_sub(1)?)
        .map(str::len)
        .sum();
    let chunk = text[start..].split_inclusive('\n').next()?;
    let body = chunk.strip_suffix('\n').unwrap_or(chunk);
    let body = body.strip_suffix('\r').unwrap_or(body);
    Some(format!(
        "{}{}{}",
        &text[..start],
        f(body),
        &text[start + body.len()..]
    ))
}

/// Interactive fix mode controller
pub struct InteractiveFixer {
    violations: Vec<FixableViolation>,
    current_index: usize,
    auto_apply_all: bool,
}

impl InteractiveFixer {
    pub const fn new() -> Self {
        Self {
            violations: Vec::new(),
            current_index: 0,
            auto_apply_all: false,
        }
    }

    /// Collect all fixable violations from files
    pub fn collect_violations(
        &mut self,
        paths: &[Utf8PathBuf],
        changed: Option<&crate::diff::ChangedLines>,
        config: Option<&DictateConfig>,
    ) -> Result<()> {
        let mut files = crate::files::collect_all_files(paths)?;
        if let Some(changed) = changed {
            files.retain(|f| changed.contains_file(f));
        }
        let file_types = crate::files::detect_file_types(&files);
        let mut regime = crate::regime::init_regime_for_files(&file_types, config);

        // Load custom WASM decrees from config
        if let Some(cfg) = config {
            for settings in cfg.decree.values() {
                if settings.path.is_some()
                    && settings.enabled.unwrap_or(true)
                    && let Some(ref path) = settings.path
                {
                    regime.add_wasm_decree(path)?;
                }
            }
        }

        let scope = changed.map(|c| crate::diff::Scope::new(c.clone(), regime.file_scope_rules()));

        for path in files {
            let Some(text) = crate::files::read_source_file(path.as_std_path()) else {
                continue;
            };
            let source = dictator_core::Source {
                path: path.as_path(),
                text: &text,
            };

            let diags = regime.enforce(&[source])?;

            for diag in diags {
                if scope
                    .as_ref()
                    .is_some_and(|s| !s.allows(path.as_path(), &text, &diag))
                {
                    continue;
                }
                if diag.enforced {
                    let (line, column) = crate::output::byte_to_line_col(&text, diag.span.start);
                    let violation = FixableViolation {
                        path: path.clone(),
                        line,
                        column,
                        rule: diag.rule,
                        message: diag.message,
                    };
                    if violation.fix(&text).is_some() {
                        self.violations.push(violation);
                    }
                }
            }
        }

        Ok(())
    }

    /// Run interactive fix mode
    pub fn run_interactive(&mut self, auto_apply_all: bool) -> Result<()> {
        self.auto_apply_all = auto_apply_all;

        if self.violations.is_empty() {
            println!("✨ No fixable violations found!");
            return Ok(());
        }

        println!("🔧 Found {} fixable violations", self.violations.len());
        println!("Commands: [f]ix [s]kip [a]ll [q]uit [d]etails [h]elp\n");

        let mut applied_count = 0;
        let mut skipped_count = 0;

        while self.current_index < self.violations.len() {
            let violation = &self.violations[self.current_index];

            // Re-read: fixes accepted earlier may have rewritten this file.
            let text = fs::read_to_string(&violation.path)?;
            let Some((fixed, description)) = violation.fix(&text) else {
                // An earlier fix on the same line already settled it.
                self.current_index += 1;
                continue;
            };

            if self.auto_apply_all {
                Self::apply_fix(violation, &fixed)?;
                applied_count += 1;
                self.current_index += 1;
                continue;
            }

            Self::show_violation(violation, &text, &fixed, description);

            match Self::get_user_choice()? {
                UserChoice::Fix => {
                    Self::apply_fix(violation, &fixed)?;
                    applied_count += 1;
                    self.current_index += 1;
                }
                UserChoice::Skip => {
                    skipped_count += 1;
                    self.current_index += 1;
                }
                UserChoice::All => {
                    self.auto_apply_all = true;
                    // Continue to apply this fix and all remaining
                }
                UserChoice::Quit => {
                    println!("\n👋 Exiting interactive fix mode");
                    break;
                }
                UserChoice::Details => {
                    Self::show_details(violation, &text, description);
                }
                UserChoice::Help => {
                    Self::show_help();
                }
            }
        }

        println!("\n📊 Summary:");
        println!("  Applied fixes: {applied_count}");
        println!("  Skipped fixes: {skipped_count}");
        println!("  Total processed: {}", applied_count + skipped_count);

        Ok(())
    }

    /// Show current violation to user
    fn show_violation(violation: &FixableViolation, text: &str, fixed: &str, description: &str) {
        println!(
            "📍 {}:{}:{}",
            violation.path, violation.line, violation.column
        );
        println!("   Rule: {}", violation.rule);
        println!("   Issue: {}", violation.message);
        println!("   Fix: {description}");

        // Show a diff of the change
        let original_line = text.lines().nth(violation.line - 1).unwrap_or("");
        let fixed_line = fixed.lines().nth(violation.line - 1).unwrap_or("");

        if original_line != fixed_line {
            println!("   --- {} ---", violation.path);
            println!("   - {original_line}");
            println!("   + {fixed_line}");
        }

        print!("   Choice [>f/s/a/q/d/h>]: ");
        io::stdout().flush().unwrap();
    }

    /// Get user choice
    fn get_user_choice() -> Result<UserChoice> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(match input.trim().to_lowercase().as_str() {
            "f" | "" | "fix" => UserChoice::Fix,
            "s" | "skip" => UserChoice::Skip,
            "a" | "all" => UserChoice::All,
            "q" | "quit" => UserChoice::Quit,
            "d" | "details" => UserChoice::Details,
            "h" | "help" => UserChoice::Help,
            _ => {
                println!("   Invalid choice. Please try again.");
                Self::get_user_choice()?
            }
        })
    }

    /// Show detailed information about the violation
    fn show_details(violation: &FixableViolation, text: &str, description: &str) {
        println!("\n   📋 Detailed Information:");
        println!("      File: {}", violation.path);
        println!("      Position: {}:{}", violation.line, violation.column);
        println!("      Rule: {}", violation.rule);
        println!("      Message: {}", violation.message);
        println!("      Description: {description}");

        // Show more context around the violation
        let context_lines = 3;
        let lines: Vec<&str> = text.lines().collect();
        let start_line = violation.line.saturating_sub(context_lines);
        let end_line = (violation.line + context_lines).min(lines.len());

        println!("      Context:");
        for (i, line) in lines
            .iter()
            .enumerate()
            .skip(start_line)
            .take(end_line - start_line)
        {
            let marker = if i + 1 == violation.line { ">>" } else { "  " };
            println!("      {} {:4} | {}", marker, i + 1, line);
        }
        println!();
    }

    /// Show help information
    fn show_help() {
        println!("\n   📖 Interactive Fix Mode Help:");
        println!("      f/Enter - Apply this fix");
        println!("      s       - Skip this fix");
        println!("      a       - Apply all remaining fixes automatically");
        println!("      q       - Quit interactive mode");
        println!("      d       - Show detailed information");
        println!("      h       - Show this help");
        println!();
    }

    /// Apply a fix to the file
    fn apply_fix(violation: &FixableViolation, fixed: &str) -> Result<()> {
        fs::write(&violation.path, fixed)?;
        println!("   ✅ Applied fix to {}", violation.path);
        Ok(())
    }
}

/// User choices in interactive mode
#[derive(Debug, Clone, Copy)]
enum UserChoice {
    Fix,
    Skip,
    All,
    Quit,
    Details,
    Help,
}

#[cfg(test)]
mod tests;
