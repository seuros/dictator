//! Interactive fix mode - allows users to review and selectively apply fixes

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::DictateConfig;
use std::fs;
use std::io::{self, Write};

/// Represents a fixable violation with context
#[derive(Debug, Clone)]
pub struct FixableViolation {
    pub path: Utf8PathBuf,
    pub line: usize,
    pub column: usize,
    pub rule: String,
    pub message: String,
    pub original_text: String,
    pub fixed_text: String,
    pub description: String,
}

/// Interactive fix mode controller
pub struct InteractiveFixer {
    violations: Vec<FixableViolation>,
    current_index: usize,
    auto_apply_all: bool,
}

impl InteractiveFixer {
    /// Get the number of violations found
    pub const fn violation_count(&self) -> usize {
        self.violations.len()
    }

    /// Check if there are any violations
    pub const fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

impl InteractiveFixer {
    pub const fn new() -> Self {
        Self { violations: Vec::new(), current_index: 0, auto_apply_all: false }
    }

    /// Collect all fixable violations from files
    pub fn collect_violations(
        &mut self,
        paths: &[Utf8PathBuf],
        config: Option<&DictateConfig>,
    ) -> Result<()> {
        let files = crate::files::collect_all_files(paths)?;
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

        for path in files {
            let Some(text) = crate::files::read_source_file(path.as_std_path()) else {
                continue;
            };
            let source = dictator_core::Source { path: path.as_path(), text: &text };

            let diags = regime.enforce(&[source])?;

            for diag in diags {
                if diag.enforced {
                    // This is a fixable violation
                    if let Some(fix) = Self::create_fix(&path, &text, &diag) {
                        self.violations.push(fix);
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a fix for a specific violation
    fn create_fix(
        path: &Utf8PathBuf,
        text: &str,
        diag: &dictator_decree_abi::Diagnostic,
    ) -> Option<FixableViolation> {
        let (line, column) = crate::output::byte_to_line_col(text, diag.span.start);

        // Get the line containing the violation
        let lines: Vec<&str> = text.lines().collect();
        if line > lines.len() {
            return None;
        }

        let original_line = lines[line - 1];

        // Apply the fix based on the rule
        let (fixed_line, description): (String, &str) = match diag.rule.as_str() {
            rule if rule.contains("trailing-whitespace") => {
                (original_line.trim_end().to_string(), "Remove trailing whitespace")
            }
            rule if rule.contains("tab-character") => {
                (original_line.replace('\t', "  "), "Replace tabs with spaces")
            }
            rule if rule.contains("missing-final-newline") => {
                // This is a file-level fix, not line-level
                return Some(FixableViolation {
                    path: path.clone(),
                    line,
                    column,
                    rule: diag.rule.clone(),
                    message: diag.message.clone(),
                    original_text: text.to_string(),
                    fixed_text: if text.ends_with('\n') {
                        text.to_string()
                    } else {
                        format!("{text}\n")
                    },
                    description: "Add final newline".to_string(),
                });
            }
            _ => return None, // Not a fixable rule we handle
        };

        if fixed_line == original_line {
            return None; // No change needed
        }

        // Reconstruct the full text with the fix
        let mut lines_fixed = lines.clone();
        lines_fixed[line - 1] = &fixed_line;
        let fixed_text = lines_fixed.join("\n");

        Some(FixableViolation {
            path: path.clone(),
            line,
            column,
            rule: diag.rule.clone(),
            message: diag.message.clone(),
            original_text: text.to_string(),
            fixed_text,
            description: description.to_string(),
        })
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

            if self.auto_apply_all {
                Self::apply_fix(violation)?;
                applied_count += 1;
                self.current_index += 1;
                continue;
            }

            Self::show_violation(violation);

            match Self::get_user_choice()? {
                UserChoice::Fix => {
                    Self::apply_fix(violation)?;
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
                    Self::show_details(violation);
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
    fn show_violation(violation: &FixableViolation) {
        println!("📍 {}:{}:{}", violation.path, violation.line, violation.column);
        println!("   Rule: {}", violation.rule);
        println!("   Issue: {}", violation.message);
        println!("   Fix: {}", violation.description);

        // Show a diff of the change
        let original_line = violation.original_text.lines().nth(violation.line - 1).unwrap_or("");
        let fixed_line = violation.fixed_text.lines().nth(violation.line - 1).unwrap_or("");

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
    fn show_details(violation: &FixableViolation) {
        println!("\n   📋 Detailed Information:");
        println!("      File: {}", violation.path);
        println!("      Position: {}:{}", violation.line, violation.column);
        println!("      Rule: {}", violation.rule);
        println!("      Message: {}", violation.message);
        println!("      Description: {}", violation.description);

        // Show more context around the violation
        let context_lines = 3;
        let lines: Vec<&str> = violation.original_text.lines().collect();
        let start_line = violation.line.saturating_sub(context_lines);
        let end_line = (violation.line + context_lines).min(lines.len());

        println!("      Context:");
        for (i, line) in lines.iter().enumerate().skip(start_line).take(end_line - start_line) {
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
    fn apply_fix(violation: &FixableViolation) -> Result<()> {
        fs::write(&violation.path, &violation.fixed_text)?;
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
mod tests {
    use super::*;

    #[test]
    fn test_interactive_fixer_creation() {
        let fixer = InteractiveFixer::new();
        assert_eq!(fixer.violations.len(), 0);
        assert_eq!(fixer.current_index, 0);
        assert!(!fixer.auto_apply_all);
    }
}
