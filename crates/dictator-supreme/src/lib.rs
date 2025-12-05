#![warn(rust_2024_compatibility, clippy::all)]

//! decree.supreme - Universal structural rules for ALL files.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;
use std::collections::HashMap;

/// Configuration for supreme decree (will be loaded from .dictate.toml)
#[derive(Debug, Clone)]
pub struct SupremeConfig {
    pub max_line_length: Option<usize>,
    pub trailing_whitespace: bool,
    pub tabs_vs_spaces: TabsOrSpaces,
    pub final_newline: bool,
    pub blank_line_whitespace: bool,
    pub line_endings: LineEnding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsOrSpaces {
    Tabs,
    Spaces,
    Either, // Don't enforce
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,     // Unix
    Crlf,   // Windows
    Either, // Don't enforce
}

impl Default for SupremeConfig {
    fn default() -> Self {
        Self {
            max_line_length: None, // Opt-in per language
            trailing_whitespace: true,
            tabs_vs_spaces: TabsOrSpaces::Spaces,
            final_newline: true,
            blank_line_whitespace: true,
            line_endings: LineEnding::Lf,
        }
    }
}

/// Lint source for universal structural violations.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_config(source, &SupremeConfig::default())
}

#[must_use]
pub fn lint_source_with_config(source: &str, config: &SupremeConfig) -> Diagnostics {
    lint_source_with_owner(source, config, "supreme")
}

/// Lint with configurable rule owner - "you touch it, you own it"
#[must_use]
pub fn lint_source_with_owner(source: &str, config: &SupremeConfig, owner: &str) -> Diagnostics {
    let mut diags = Diagnostics::new();
    let bytes = source.as_bytes();

    // Detect line ending type and check for mixed endings
    let line_ending_info = detect_line_endings(bytes);
    if line_ending_info.has_mixed && config.line_endings != LineEnding::Either {
        diags.push(Diagnostic {
            rule: format!("{owner}/mixed-line-endings"),
            message: format!(
                "{} CRLF, {} LF",
                line_ending_info.crlf_count, line_ending_info.lf_count
            ),
            enforced: true,
            span: Span::new(0, bytes.len().min(100)),
        });
    }

    // Check expected line ending type
    if config.line_endings != LineEnding::Either {
        match (config.line_endings, &line_ending_info) {
            (LineEnding::Lf, info) if info.crlf_count > 0 && !info.has_mixed => {
                diags.push(Diagnostic {
                    rule: format!("{owner}/wrong-line-ending"),
                    message: "CRLF, expected LF".to_string(),
                    enforced: true,
                    span: Span::new(0, bytes.len().min(100)),
                });
            }
            (LineEnding::Crlf, info) if info.lf_only_count > 0 && !info.has_mixed => {
                diags.push(Diagnostic {
                    rule: format!("{owner}/wrong-line-ending"),
                    message: "LF, expected CRLF".to_string(),
                    enforced: true,
                    span: Span::new(0, bytes.len().min(100)),
                });
            }
            _ => {}
        }
    }

    // Check each line for structural issues
    let mut line_start: usize = 0;
    let mut line_idx: usize = 0;

    for nl in memchr_iter(b'\n', bytes) {
        check_line(
            source, line_start, nl, true, line_idx, config, owner, &mut diags,
        );
        line_start = nl + 1;
        line_idx += 1;
    }

    // Handle last line without newline
    if line_start < bytes.len() {
        check_line(
            source,
            line_start,
            bytes.len(),
            false,
            line_idx,
            config,
            owner,
            &mut diags,
        );

        // Missing final newline
        if config.final_newline {
            diags.push(Diagnostic {
                rule: format!("{owner}/missing-final-newline"),
                message: "no final newline".to_string(),
                enforced: true,
                span: Span::new(bytes.len().saturating_sub(1), bytes.len()),
            });
        }
    }

    diags
}

#[derive(Debug)]
struct LineEndingInfo {
    crlf_count: usize,
    lf_only_count: usize,
    lf_count: usize,
    has_mixed: bool,
}

fn detect_line_endings(bytes: &[u8]) -> LineEndingInfo {
    let mut crlf_count = 0;
    let mut lf_only_count = 0;

    for nl_pos in memchr_iter(b'\n', bytes) {
        if nl_pos > 0 && bytes[nl_pos - 1] == b'\r' {
            crlf_count += 1;
        } else {
            lf_only_count += 1;
        }
    }

    let lf_count = crlf_count + lf_only_count;
    let has_mixed = crlf_count > 0 && lf_only_count > 0;

    LineEndingInfo {
        crlf_count,
        lf_only_count,
        lf_count,
        has_mixed,
    }
}

#[allow(clippy::too_many_arguments)]
fn check_line(
    source: &str,
    start: usize,
    end: usize,
    _had_newline: bool,
    _line_idx: usize,
    config: &SupremeConfig,
    owner: &str,
    diags: &mut Diagnostics,
) {
    let line = &source[start..end];

    // Strip CRLF if present
    let line = line.strip_suffix('\r').unwrap_or(line);

    // 1. Trailing whitespace
    if config.trailing_whitespace {
        let trimmed_end = line.trim_end_matches([' ', '\t']).len();
        if trimmed_end != line.len() {
            diags.push(Diagnostic {
                rule: format!("{owner}/trailing-whitespace"),
                message: "trailing whitespace".to_string(),
                enforced: true,
                span: Span::new(start + trimmed_end, start + line.len()),
            });
        }
    }

    // 2. Tabs vs Spaces
    match config.tabs_vs_spaces {
        TabsOrSpaces::Spaces => {
            if let Some(pos) = line.bytes().position(|b| b == b'\t') {
                diags.push(Diagnostic {
                    rule: format!("{owner}/tab-character"),
                    message: "tab found".to_string(),
                    enforced: true,
                    span: Span::new(start + pos, start + pos + 1),
                });
            }
        }
        TabsOrSpaces::Tabs => {
            // Check for any spaces in the indentation prefix (must be tabs only)
            if let Some((idx, _)) = line
                .char_indices()
                .take_while(|(_, c)| c.is_whitespace() && *c != '\n' && *c != '\r')
                .find(|(_, c)| *c == ' ')
            {
                diags.push(Diagnostic {
                    rule: format!("{owner}/space-indentation"),
                    message: "spaces found, use tabs".to_string(),
                    enforced: true,
                    span: Span::new(start + idx, start + idx + 1),
                });
            }
        }
        TabsOrSpaces::Either => {
            // Don't enforce
        }
    }

    // 3. Blank line with whitespace
    if config.blank_line_whitespace && line.trim().is_empty() && !line.is_empty() {
        diags.push(Diagnostic {
            rule: format!("{owner}/blank-line-whitespace"),
            message: "blank line has whitespace".to_string(),
            enforced: true,
            span: Span::new(start, start + line.len()),
        });
    }

    // 4. Line length (opt-in per language)
    if let Some(max_len) = config.max_line_length
        && line.len() > max_len
    {
        diags.push(Diagnostic {
            rule: format!("{owner}/line-too-long"),
            message: format!("{} > {}", line.len(), max_len),
            enforced: true,
            span: Span::new(start, start + line.len()),
        });
    }
}

/// Map file extension to language decree name
fn ext_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rb" | "rake" | "gemspec" | "ru" => Some("ruby"),
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => Some("typescript"),
        "go" => Some("golang"),
        "rs" => Some("rust"),
        "py" | "pyi" => Some("python"),
        _ => None,
    }
}

#[derive(Default)]
pub struct Supreme {
    config: SupremeConfig,
    /// Language-specific overrides (language name -> partial config)
    language_overrides: HashMap<String, SupremeConfig>,
}

impl Supreme {
    #[must_use]
    pub fn new(config: SupremeConfig) -> Self {
        Self {
            config,
            language_overrides: HashMap::new(),
        }
    }

    /// Create with language overrides
    #[must_use]
    pub const fn with_language_overrides(
        config: SupremeConfig,
        overrides: HashMap<String, SupremeConfig>,
    ) -> Self {
        Self {
            config,
            language_overrides: overrides,
        }
    }

    /// Get effective config and rule owner for a file path
    /// Returns (config, owner) where owner is the language name if overridden, else "supreme"
    fn config_for_path(&self, path: &str) -> (SupremeConfig, &str) {
        // Extract extension from path
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Look up language override - "you touch it, you own it"
        if let Some(lang) = ext_to_language(ext)
            && let Some(override_config) = self.language_overrides.get(lang)
        {
            return (override_config.clone(), lang);
        }

        // No override, supreme owns it
        (self.config.clone(), "supreme")
    }
}

impl Decree for Supreme {
    fn name(&self) -> &'static str {
        "supreme"
    }

    fn lint(&self, path: &str, source: &str) -> Diagnostics {
        let (effective_config, owner) = self.config_for_path(path);
        lint_source_with_owner(source, &effective_config, owner)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Supreme structural rules (universal)".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec![],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(Supreme::default())
}

/// Create plugin with custom config
#[must_use]
pub fn init_decree_with_config(config: SupremeConfig) -> BoxDecree {
    Box::new(Supreme::new(config))
}

/// Create plugin with config and language overrides
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn init_decree_with_overrides(
    config: SupremeConfig,
    overrides: HashMap<String, SupremeConfig>,
) -> BoxDecree {
    Box::new(Supreme::with_language_overrides(config, overrides))
}

/// Convert `DecreeSettings` to `SupremeConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> SupremeConfig {
    SupremeConfig {
        max_line_length: settings.max_line_length,
        trailing_whitespace: settings
            .trailing_whitespace
            .as_deref()
            .is_none_or(|s| s == "deny"),
        tabs_vs_spaces: settings.tabs_vs_spaces.as_deref().map_or(
            TabsOrSpaces::Spaces,
            |s| match s {
                "tabs" => TabsOrSpaces::Tabs,
                "spaces" => TabsOrSpaces::Spaces,
                _ => TabsOrSpaces::Either,
            },
        ),
        final_newline: settings
            .final_newline
            .as_deref()
            .is_none_or(|s| s == "require"),
        blank_line_whitespace: settings
            .blank_line_whitespace
            .as_deref()
            .is_none_or(|s| s == "deny"),
        line_endings: settings
            .line_endings
            .as_deref()
            .map_or(LineEnding::Lf, |s| match s {
                "lf" => LineEnding::Lf,
                "crlf" => LineEnding::Crlf,
                _ => LineEnding::Either,
            }),
    }
}

/// Create merged config: base supreme + language override
/// Language settings override supreme settings when explicitly set
#[must_use]
pub fn merged_config(
    base: &dictator_core::DecreeSettings,
    lang: &dictator_core::DecreeSettings,
) -> SupremeConfig {
    SupremeConfig {
        // Language overrides base if set, otherwise use base
        max_line_length: lang.max_line_length.or(base.max_line_length),
        trailing_whitespace: lang
            .trailing_whitespace
            .as_deref()
            .or(base.trailing_whitespace.as_deref())
            .is_none_or(|s| s == "deny"),
        tabs_vs_spaces: lang
            .tabs_vs_spaces
            .as_deref()
            .or(base.tabs_vs_spaces.as_deref())
            .map_or(TabsOrSpaces::Spaces, |s| match s {
                "tabs" => TabsOrSpaces::Tabs,
                "spaces" => TabsOrSpaces::Spaces,
                _ => TabsOrSpaces::Either,
            }),
        final_newline: lang
            .final_newline
            .as_deref()
            .or(base.final_newline.as_deref())
            .is_none_or(|s| s == "require"),
        blank_line_whitespace: lang
            .blank_line_whitespace
            .as_deref()
            .or(base.blank_line_whitespace.as_deref())
            .is_none_or(|s| s == "deny"),
        line_endings: lang
            .line_endings
            .as_deref()
            .or(base.line_endings.as_deref())
            .map_or(LineEnding::Lf, |s| match s {
                "lf" => LineEnding::Lf,
                "crlf" => LineEnding::Crlf,
                _ => LineEnding::Either,
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_trailing_whitespace() {
        let src = "hello world  \n";
        let diags = lint_source(src);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "supreme/trailing-whitespace")
        );
    }

    #[test]
    fn detects_tabs_when_spaces_expected() {
        let src = "hello\tworld\n";
        let diags = lint_source(src);
        assert!(diags.iter().any(|d| d.rule == "supreme/tab-character"));
    }

    #[test]
    fn allows_tabs_when_configured() {
        let src = "\thello world\n";
        let config = SupremeConfig {
            tabs_vs_spaces: TabsOrSpaces::Tabs,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(!diags.iter().any(|d| d.rule == "supreme/tab-character"));
    }

    #[test]
    fn detects_spaces_when_tabs_expected() {
        let src = "  hello world\n";
        let config = SupremeConfig {
            tabs_vs_spaces: TabsOrSpaces::Tabs,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(diags.iter().any(|d| d.rule == "supreme/space-indentation"));
    }

    #[test]
    fn detects_single_space_when_tabs_expected() {
        let src = " hello world\n";
        let config = SupremeConfig {
            tabs_vs_spaces: TabsOrSpaces::Tabs,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(diags.iter().any(|d| d.rule == "supreme/space-indentation"));
    }

    #[test]
    fn detects_mixed_tabs_and_spaces_when_tabs_expected() {
        let src = "\t hello world\n"; // tab then space
        let config = SupremeConfig {
            tabs_vs_spaces: TabsOrSpaces::Tabs,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(diags.iter().any(|d| d.rule == "supreme/space-indentation"));
    }

    #[test]
    fn detects_missing_final_newline() {
        let src = "hello world";
        let diags = lint_source(src);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "supreme/missing-final-newline")
        );
    }

    #[test]
    fn allows_missing_final_newline_when_configured() {
        let src = "hello world";
        let config = SupremeConfig {
            final_newline: false,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule == "supreme/missing-final-newline")
        );
    }

    #[test]
    fn detects_blank_line_whitespace() {
        let src = "line1\n   \nline2\n";
        let diags = lint_source(src);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "supreme/blank-line-whitespace")
        );
    }

    #[test]
    fn detects_line_too_long() {
        let src = format!("{}\n", "x".repeat(150));
        let config = SupremeConfig {
            max_line_length: Some(120),
            ..Default::default()
        };
        let diags = lint_source_with_config(&src, &config);
        assert!(diags.iter().any(|d| d.rule == "supreme/line-too-long"));
    }

    #[test]
    fn skips_line_length_when_disabled() {
        let src = format!("{}\n", "x".repeat(500));
        let diags = lint_source(&src); // Default has max_line_length: None
        assert!(!diags.iter().any(|d| d.rule == "supreme/line-too-long"));
    }

    #[test]
    fn detects_mixed_line_endings() {
        let src = "line1\r\nline2\nline3\r\n";
        let diags = lint_source(src);
        assert!(diags.iter().any(|d| d.rule == "supreme/mixed-line-endings"));
    }

    #[test]
    fn detects_crlf_when_lf_expected() {
        let src = "line1\r\nline2\r\n";
        let config = SupremeConfig {
            line_endings: LineEnding::Lf,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(diags.iter().any(|d| d.rule == "supreme/wrong-line-ending"));
    }

    #[test]
    fn detects_lf_when_crlf_expected() {
        let src = "line1\nline2\n";
        let config = SupremeConfig {
            line_endings: LineEnding::Crlf,
            ..Default::default()
        };
        let diags = lint_source_with_config(src, &config);
        assert!(diags.iter().any(|d| d.rule == "supreme/wrong-line-ending"));
    }

    #[test]
    fn handles_empty_file() {
        let src = "";
        let diags = lint_source(src);
        // Empty file is valid (has no violations except maybe missing final newline)
        assert!(diags.is_empty() || diags.len() == 1);
    }

    #[test]
    fn handles_single_line_with_newline() {
        let src = "hello world\n";
        let diags = lint_source(src);
        assert!(diags.is_empty());
    }
}
