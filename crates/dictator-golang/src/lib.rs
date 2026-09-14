#![warn(rust_2024_compatibility, clippy::all)]

//! decree.golang - Go structural rules.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use dictator_supreme::{SupremeConfig, TabsOrSpaces};
use memchr::memchr_iter;

/// Configuration for golang decree
#[derive(Debug, Clone)]
pub struct GolangConfig {
    pub max_lines: usize,
    pub ignore_comments: bool,
}

impl Default for GolangConfig {
    fn default() -> Self {
        Self { max_lines: 450, ignore_comments: false }
    }
}

/// Lint Go source for structural violations.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_configs(source, &GolangConfig::default(), &SupremeConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &GolangConfig) -> Diagnostics {
    lint_source_with_configs(source, config, &SupremeConfig::default())
}

/// Lint with custom golang config + supreme config
#[must_use]
pub fn lint_source_with_configs(
    source: &str,
    config: &GolangConfig,
    supreme_config: &SupremeConfig,
) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Go's decree owns indentation style; suppress supreme's tabs/spaces check to avoid
    // duplicate diagnostics.
    let mut supreme = supreme_config.clone();
    supreme.tabs_vs_spaces = TabsOrSpaces::Either;

    let supreme_diags = dictator_supreme::lint_source_with_owner(source, &supreme, "golang");

    if config.ignore_comments {
        diags.extend(dictator_supreme::retain_long_line_diags(
            source,
            supreme_diags,
            "golang/line-too-long",
            "//",
        ));
    } else {
        diags.extend(supreme_diags);
    }

    // Golang-specific rules
    check_file_line_count(source, config.max_lines, &mut diags);
    check_indentation_style(source, &mut diags);

    diags
}

/// Rule 1: File line count (ignoring comments and blank lines)
fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let code_lines = dictator_supreme::count_code_lines(source, is_comment_only_line);

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "golang/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines (max {max_lines}, excl. comments/blanks)"
            ),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// Check if a line is comment-only (// or /* */ style)
fn is_comment_only_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

/// Rule 2: Indentation style - Go requires tabs, not spaces
/// Skips lines inside raw string literals (backtick strings)
fn check_indentation_style(source: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut line_start = 0;
    let mut in_raw_string = false;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];

        // Count backticks in this line to track raw string state
        let backtick_count = line.bytes().filter(|&b| b == b'`').count();
        let was_in_raw_string = in_raw_string;

        // Toggle state for each backtick (odd count flips state)
        if backtick_count % 2 == 1 {
            in_raw_string = !in_raw_string;
        }

        // Skip empty lines and lines inside raw strings
        if line.trim().is_empty() || was_in_raw_string {
            line_start = nl + 1;
            continue;
        }

        // Check if line starts with spaces (not tabs)
        // Go convention: only tabs for indentation
        if line.starts_with(' ') {
            diags.push(Diagnostic {
                rule: "golang/spaces-instead-of-tabs".to_string(),
                message: "Go requires tabs for indentation, not spaces".to_string(),
                enforced: true,
                span: Span::new(line_start, nl),
            });
        }

        line_start = nl + 1;
    }

    // Handle last line without newline
    if line_start < bytes.len() {
        let line = &source[line_start..];
        if !line.trim().is_empty() && !in_raw_string && line.starts_with(' ') {
            diags.push(Diagnostic {
                rule: "golang/spaces-instead-of-tabs".to_string(),
                message: "Go requires tabs for indentation, not spaces".to_string(),
                enforced: true,
                span: Span::new(line_start, bytes.len()),
            });
        }
    }
}

#[derive(Default)]
pub struct Golang {
    config: GolangConfig,
    supreme: SupremeConfig,
}

impl Golang {
    #[must_use]
    pub const fn new(config: GolangConfig, supreme: SupremeConfig) -> Self {
        Self { config, supreme }
    }
}

impl Decree for Golang {
    fn name(&self) -> &'static str {
        "golang"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        lint_source_with_configs(source, &self.config, &self.supreme)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Go structural rules".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["go".to_string()],
            supported_filenames: vec!["go.mod".to_string(), "go.work".to_string()],
            skip_filenames: vec!["go.sum".to_string()],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(Golang::default())
}

/// Create decree with custom config
#[must_use]
pub fn init_decree_with_config(config: GolangConfig) -> BoxDecree {
    Box::new(Golang::new(config, SupremeConfig::default()))
}

/// Create decree with custom config + supreme config (merged from decree.supreme + decree.golang)
#[must_use]
pub fn init_decree_with_configs(config: GolangConfig, supreme: SupremeConfig) -> BoxDecree {
    Box::new(Golang::new(config, supreme))
}

/// Convert `DecreeSettings` to `GolangConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> GolangConfig {
    GolangConfig {
        max_lines: settings.max_lines.unwrap_or(450),
        ignore_comments: settings.ignore_comments.unwrap_or(false),
    }
}

#[cfg(test)]
mod tests;
