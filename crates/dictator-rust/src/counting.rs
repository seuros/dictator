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
mod tests {
    use crate::lint_source;

    #[test]
    fn detects_file_too_long() {
        use std::fmt::Write;
        let mut src = String::new();
        for i in 0..410 {
            let _ = writeln!(src, "let x{i} = {i};");
        }
        let diags = lint_source(&src);
        assert!(
            diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "Should detect file with >400 code lines"
        );
    }

    #[test]
    fn ignores_comments_in_line_count() {
        use std::fmt::Write;
        // 390 code lines + 60 comment lines = 450 total, but only 390 counted
        let mut src = String::new();
        for i in 0..390 {
            let _ = writeln!(src, "let x{i} = {i};");
        }
        for i in 0..60 {
            let _ = writeln!(src, "// Comment {i}");
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "Should not count comment-only lines"
        );
    }

    #[test]
    fn ignores_blank_lines_in_count() {
        use std::fmt::Write;
        // 390 code lines + 60 blank lines = 450 total, but only 390 counted
        let mut src = String::new();
        for i in 0..390 {
            let _ = writeln!(src, "let x{i} = {i};");
        }
        for _ in 0..60 {
            src.push('\n');
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "Should not count blank lines"
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
            !diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "File with only comments should not trigger line count"
        );
    }
}
