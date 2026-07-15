//! Shared text-scanning helpers reused across language decrees.
//!
//! These loops were copy-pasted into every language decree crate. Supreme is the
//! crate they all already depend on, so centralizing them here removes the
//! duplication while keeping the behavior byte-for-byte identical.

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;

/// Count non-blank, non-comment lines in `source`.
///
/// Blank lines are always excluded; `is_comment_only` decides whether a trimmed
/// line is comment-only (and therefore excluded). The final line is counted even
/// when it lacks a trailing newline.
pub fn count_code_lines(source: &str, is_comment_only: impl Fn(&str) -> bool) -> usize {
    let bytes = source.as_bytes();
    let mut code_lines = 0;
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let trimmed = source[line_start..nl].trim();
        if !trimmed.is_empty() && !is_comment_only(trimmed) {
            code_lines += 1;
        }
        line_start = nl + 1;
    }

    // Handle last line without a trailing newline.
    if line_start < bytes.len() {
        let trimmed = source[line_start..].trim();
        if !trimmed.is_empty() && !is_comment_only(trimmed) {
            code_lines += 1;
        }
    }

    code_lines
}

/// Drop `line-too-long` diagnostics that land on comment-only lines.
///
/// `rule` is the fully-qualified rule id (e.g. `"rust/line-too-long"`) and
/// `comment_prefix` is the language's line-comment marker (`"//"`, `"#"`, ...).
/// Diagnostics with any other rule are passed through untouched.
#[must_use]
pub fn retain_long_line_diags(
    source: &str,
    diags: Diagnostics,
    rule: &str,
    comment_prefix: &str,
) -> Diagnostics {
    let lines: Vec<&str> = source.lines().collect();
    diags
        .into_iter()
        .filter(|d| {
            if d.rule == rule {
                let line_idx = source[..d.span.start].matches('\n').count();
                !lines
                    .get(line_idx)
                    .is_some_and(|line| line.trim_start().starts_with(comment_prefix))
            } else {
                true
            }
        })
        .collect()
}

/// Check tab/space consistency and indentation-depth regularity.
///
/// Emits `{rule_prefix}/mixed-indentation` when both tabs and spaces are used
/// for indentation, and `{rule_prefix}/inconsistent-indentation` on the first
/// irregular depth change (not a multiple of 2 or 4 spaces).
pub fn check_indentation_consistency(source: &str, rule_prefix: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut line_start = 0;
    let mut has_tabs = false;
    let mut has_spaces = false;
    let mut inconsistent_depths: Vec<(usize, usize)> = Vec::new();
    let mut indent_stack: Vec<usize> = Vec::new();

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];

        // Skip empty lines
        if line.trim().is_empty() {
            line_start = nl + 1;
            continue;
        }

        // Detect tabs vs spaces
        if line.starts_with('\t') {
            has_tabs = true;
        } else if line.starts_with(' ') {
            has_spaces = true;
        }

        // Calculate indentation depth
        let indent = count_leading_whitespace(line);
        if indent > 0 && !line.trim().is_empty() {
            // Check for inconsistent indentation depth changes
            if let Some(&last_indent) = indent_stack.last() {
                if indent > last_indent {
                    // Indentation increased
                    let diff = indent - last_indent;
                    // Check if it's a consistent multiple (2 or 4 spaces, or 1 tab)
                    if has_spaces && diff != 2 && diff != 4 {
                        inconsistent_depths.push((line_start, nl));
                    }
                    indent_stack.push(indent);
                } else if indent < last_indent {
                    // Indentation decreased - pop stack until we find matching level
                    while let Some(&stack_indent) = indent_stack.last() {
                        if stack_indent <= indent {
                            break;
                        }
                        indent_stack.pop();
                    }
                    // If current indent doesn't match any previous level, it's inconsistent
                    if indent_stack.last() != Some(&indent) && indent > 0 {
                        inconsistent_depths.push((line_start, nl));
                    }
                }
            } else if indent > 0 {
                indent_stack.push(indent);
            }
        }

        line_start = nl + 1;
    }

    // Handle last line without newline
    if line_start < bytes.len() {
        let line = &source[line_start..];
        if !line.trim().is_empty() {
            if line.starts_with('\t') {
                has_tabs = true;
            } else if line.starts_with(' ') {
                has_spaces = true;
            }
        }
    }

    // Report mixed tabs and spaces
    if has_tabs && has_spaces {
        diags.push(Diagnostic {
            rule: format!("{rule_prefix}/mixed-indentation"),
            message: "File has mixed tabs and spaces for indentation".to_string(),
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }

    // Report inconsistent indentation depths
    if !inconsistent_depths.is_empty() {
        let (start, end) = inconsistent_depths[0];
        diags.push(Diagnostic {
            rule: format!("{rule_prefix}/inconsistent-indentation"),
            message: "Inconsistent indentation depth detected".to_string(),
            enforced: true,
            span: Span::new(start, end),
        });
    }
}

fn count_leading_whitespace(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace() && *c != '\n' && *c != '\r').count()
}
