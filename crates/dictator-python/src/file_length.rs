//! File length checks for Python sources.

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;

/// Default maximum allowed code lines per file (excluding comments and blanks).
pub const DEFAULT_MAX_LINES: usize = 380;

pub fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let mut code_lines = 0;
    let bytes = source.as_bytes();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        if !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            code_lines += 1;
        }

        line_start = nl + 1;
    }

    if line_start < bytes.len() {
        let line = &source[line_start..];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            code_lines += 1;
        }
    }

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "python/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines (max {max_lines}, excluding comments and blank lines)"
            ),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

fn is_comment_only_line(trimmed: &str) -> bool {
    trimmed.starts_with('#')
}
