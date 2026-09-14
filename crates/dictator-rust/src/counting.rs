//! File line counting (excluding comments and blank lines).

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// Rule 1: File line count (ignoring comments and blank lines)
pub fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let code_lines = dictator_supreme::count_code_lines(source, is_comment_only_line);

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "rust/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines (max {max_lines}, excl. comments/blanks)"
            ),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// Check if a line is comment-only (// or /* */ style)
pub fn is_comment_only_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

#[cfg(test)]
mod tests;
