//! File length checks for Python sources.

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// Default maximum allowed code lines per file (excluding comments and blanks).
pub const DEFAULT_MAX_LINES: usize = 380;

pub fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let code_lines = dictator_supreme::count_code_lines(source, |trimmed| trimmed.starts_with('#'));

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "python/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines \
                 (max {max_lines}, excluding comments and blank lines)"
            ),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}
