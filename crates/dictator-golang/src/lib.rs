#![warn(rust_2024_compatibility, clippy::all)]

//! decree.golang - Go structural rules.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;

/// Lint Go source for structural violations.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    let mut diags = Diagnostics::new();

    check_file_line_count(source, &mut diags);
    check_indentation_style(source, &mut diags);

    diags
}

/// Rule 1: File line count - max 450 lines (ignoring comments and blank lines)
fn check_file_line_count(source: &str, diags: &mut Diagnostics) {
    let mut code_lines = 0;
    let bytes = source.as_bytes();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        // Count line if it's not blank and not a comment-only line
        if !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            code_lines += 1;
        }

        line_start = nl + 1;
    }

    // Handle last line without newline
    if line_start < bytes.len() {
        let line = &source[line_start..];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            code_lines += 1;
        }
    }

    if code_lines > 450 {
        diags.push(Diagnostic {
            rule: "golang/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines (max 450, excluding comments and blank lines)"
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
fn check_indentation_style(source: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];

        // Skip empty lines
        if line.trim().is_empty() {
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
        if !line.trim().is_empty() && line.starts_with(' ') {
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
pub struct Golang;

impl Decree for Golang {
    fn name(&self) -> &'static str {
        "golang"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        lint_source(source)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Go structural rules".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["go".to_string()],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(Golang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_file_too_long() {
        use std::fmt::Write;
        // Create a file with 460 code lines (excluding blank lines and comments)
        let mut src = String::new();
        for i in 0..460 {
            let _ = writeln!(src, "x{i} := {i}");
        }
        let diags = lint_source(&src);
        assert!(
            diags.iter().any(|d| d.rule == "golang/file-too-long"),
            "Should detect file with >450 code lines"
        );
    }

    #[test]
    fn ignores_comments_in_line_count() {
        use std::fmt::Write;
        // 440 code lines + 60 comment lines = 500 total, but only 440 counted
        let mut src = String::new();
        for i in 0..440 {
            let _ = writeln!(src, "x{i} := {i}");
        }
        for i in 0..60 {
            let _ = writeln!(src, "// Comment {i}");
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "golang/file-too-long"),
            "Should not count comment-only lines"
        );
    }

    #[test]
    fn ignores_blank_lines_in_count() {
        use std::fmt::Write;
        // 440 code lines + 60 blank lines = 500 total, but only 440 counted
        let mut src = String::new();
        for i in 0..440 {
            let _ = writeln!(src, "x{i} := {i}");
        }
        for _ in 0..60 {
            src.push('\n');
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "golang/file-too-long"),
            "Should not count blank lines"
        );
    }

    #[test]
    fn detects_spaces_instead_of_tabs() {
        let src = "package main\n\nfunc test() {\n    x := 1\n}\n";
        let diags = lint_source(src);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
            "Should detect spaces used for indentation"
        );
    }

    #[test]
    fn allows_tabs_for_indentation() {
        let src = "package main\n\nfunc test() {\n\tx := 1\n}\n";
        let diags = lint_source(src);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
            "Should allow tabs for indentation"
        );
    }

    #[test]
    fn detects_spaces_at_line_start() {
        let src = "package main\n\nfunc test() {\n    \tx := 1\n}\n";
        let diags = lint_source(src);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
            "Should detect spaces at start of indented line"
        );
    }

    #[test]
    fn handles_empty_file() {
        let src = "";
        let diags = lint_source(src);
        assert!(diags.is_empty(), "Empty file should have no violations");
    }

    #[test]
    fn handles_file_with_only_comments() {
        let src = "// Comment 1\n// Comment 2\n/* Block comment */\n";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "golang/file-too-long"),
            "File with only comments should not trigger line count"
        );
    }

    #[test]
    fn allows_blank_lines() {
        let src = "package main\n\n\nfunc test() {\n\tx := 1\n}\n";
        let diags = lint_source(src);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
            "Should allow blank lines"
        );
    }
}
