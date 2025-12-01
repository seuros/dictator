//! Ruby hygiene rules implemented as a Dictator decree.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;

/// Configuration for ruby decree
#[derive(Debug, Clone)]
pub struct RubyConfig {
    pub max_lines: usize,
}

impl Default for RubyConfig {
    fn default() -> Self {
        Self { max_lines: 300 }
    }
}

/// Lint a Ruby source file and emit diagnostics for common hygiene issues.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_config(source, &RubyConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &RubyConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Check file line count
    check_file_line_count(source, config.max_lines, &mut diags);

    let bytes = source.as_bytes();
    let mut line_start: usize = 0;
    let mut line_idx: usize = 0;

    for nl in memchr_iter(b'\n', bytes) {
        process_line(source, line_start, nl, true, line_idx, &mut diags);
        line_start = nl + 1;
        line_idx += 1;
    }

    if line_start < bytes.len() {
        // Final line without trailing newline.
        process_line(source, line_start, bytes.len(), false, line_idx, &mut diags);
        diags.push(Diagnostic {
            rule: "ruby/missing-final-newline".to_string(),
            message: "File should end with a single newline".to_string(),
            enforced: true,
            span: Span::new(bytes.len().saturating_sub(1), bytes.len()),
        });
    }

    diags
}

/// Check file line count (excluding comments and blank lines)
fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let mut code_lines = 0;
    let bytes = source.as_bytes();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        // Count line if it's not blank and not a comment-only line
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            code_lines += 1;
        }

        line_start = nl + 1;
    }

    // Handle last line without newline
    if line_start < bytes.len() {
        let line = &source[line_start..];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            code_lines += 1;
        }
    }

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "ruby/file-too-long".to_string(),
            message: format!("{code_lines} code lines (max {max_lines})"),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

#[derive(Default)]
pub struct RubyHygiene {
    config: RubyConfig,
}

impl RubyHygiene {
    #[must_use]
    pub const fn new(config: RubyConfig) -> Self {
        Self { config }
    }
}

impl Decree for RubyHygiene {
    fn name(&self) -> &'static str {
        "ruby"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        lint_source_with_config(source, &self.config)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Ruby code structure and hygiene".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["rb".to_string(), "rake".to_string(), "gemspec".to_string()],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

/// Factory used by host (native or WASM-exported).
#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(RubyHygiene::default())
}

/// Create plugin with custom config
#[must_use]
pub fn init_decree_with_config(config: RubyConfig) -> BoxDecree {
    Box::new(RubyHygiene::new(config))
}

/// Convert `DecreeSettings` to `RubyConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> RubyConfig {
    RubyConfig {
        max_lines: settings.max_lines.unwrap_or(300),
    }
}

fn process_line(
    source: &str,
    start: usize,
    end: usize,
    had_newline: bool,
    line_idx: usize,
    diags: &mut Diagnostics,
) {
    let line = &source[start..end];

    // 1) Trailing whitespace (spaces or tabs) before the newline/end.
    let trimmed_end = line.trim_end_matches([' ', '\t']).len();
    if trimmed_end != line.len() {
        diags.push(Diagnostic {
            rule: "ruby/trailing-whitespace".to_string(),
            message: "Remove trailing whitespace".to_string(),
            enforced: true,
            span: Span::new(start + trimmed_end, start + line.len()),
        });
    }

    // 2) Tabs anywhere in the line (Ruby style guides prefer spaces).
    if let Some(pos) = line.bytes().position(|b| b == b'\t') {
        diags.push(Diagnostic {
            rule: "ruby/tab-indent".to_string(),
            message: "Use spaces instead of tabs".to_string(),
            enforced: true,
            span: Span::new(start + pos, start + pos + 1),
        });
    }

    // 2b) Whitespace-only blank lines (spaces/tabs) instead of empty newline.
    if line.trim().is_empty() && !line.is_empty() {
        diags.push(Diagnostic {
            rule: "ruby/blank-line-whitespace".to_string(),
            message: "Blank lines should not contain spaces or tabs".to_string(),
            enforced: true,
            span: Span::new(start, end),
        });
    }

    // 3) Comment hygiene: ensure space after '#', except for known directives.
    let trimmed = line.trim_start_matches(' ');
    if let Some(stripped) = trimmed.strip_prefix('#')
        && !is_comment_directive(stripped, line_idx)
        && !stripped.starts_with(' ')
        && !stripped.is_empty()
    {
        // Span of the leading '#'
        let hash_offset = start + (line.len() - trimmed.len());
        diags.push(Diagnostic {
            rule: "ruby/comment-space".to_string(),
            message: "Comments should start with '# '".to_string(),
            enforced: true,
            span: Span::new(hash_offset, hash_offset + 1),
        });
    }

    // 4) Blank line should be exactly newline (no spaces).
    if line.is_empty() && !had_newline {
        // Nothing to do for final empty slice.
    }
}

fn is_comment_directive(rest: &str, line_idx: usize) -> bool {
    let rest = rest.trim_start();

    rest.starts_with('!') // shebang
        || rest.starts_with("encoding")
        || rest.starts_with("frozen_string_literal")
        || rest.starts_with("rubocop")
        || rest.starts_with("typed")
        || (line_idx == 0 && rest.starts_with(" language"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_trailing_whitespace_and_tab() {
        let src = "def foo\n  bar \t\nend\n";
        let diags = lint_source(src);
        assert!(diags.iter().any(|d| d.rule == "ruby/trailing-whitespace"));
        assert!(diags.iter().any(|d| d.rule == "ruby/tab-indent"));
    }

    #[test]
    fn detects_missing_final_newline() {
        let src = "class Foo\nend";
        let diags = lint_source(src);
        assert!(diags.iter().any(|d| d.rule == "ruby/missing-final-newline"));
    }

    #[test]
    fn enforces_comment_space() {
        let src = "#bad\n# good\n";
        let diags = lint_source(src);
        assert!(diags.iter().any(|d| d.rule == "ruby/comment-space"));
    }

    #[test]
    fn detects_whitespace_only_blank_line() {
        let src = "def foo\n  bar\n    \nend\n"; // blank line has spaces
        let diags = lint_source(src);
        assert!(diags.iter().any(|d| d.rule == "ruby/blank-line-whitespace"));
    }
}
