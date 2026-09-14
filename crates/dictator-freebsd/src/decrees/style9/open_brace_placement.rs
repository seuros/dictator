use crate::decrees::support::*;
use crate::{FreeBsdDecree, code_line_mask};
use dictator_decree_abi::Diagnostics;

/// For conditionals (if/while/for/switch), the opening `{` must be on the same
/// line as the condition, not on its own line.
///
/// Flags:
/// ```c
/// if (cond)   // ← flagged: { should be here
/// {
/// ```
pub(crate) fn check_open_brace_placement(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let lines: Vec<&str> = source.lines().collect();
    let code_mask = code_line_mask(source);

    let offsets: Vec<usize> = {
        let mut v = Vec::with_capacity(lines.len());
        let mut off = 0usize;
        for line in &lines {
            v.push(off);
            off += line.len() + 1;
        }
        v
    };

    for i in 0..lines.len().saturating_sub(1) {
        if !code_mask[i] {
            continue;
        }
        let trimmed = lines[i].trim();
        if trimmed.starts_with("//") {
            continue;
        }

        // Look ahead: is the very next code line a lone `{`?
        let mut next_code = i + 1;
        while next_code < lines.len() && !code_mask[next_code] {
            next_code += 1;
        }
        if next_code >= lines.len() {
            continue;
        }
        let next_trimmed = lines[next_code].trim();
        if next_trimmed != "{" {
            continue;
        }

        // Mirror checkstyle9: only complain when previous line isn't already too long.
        let current_len = expanded_len(lines[i]);
        let is_conditional = trimmed.ends_with(')')
            && (trimmed.contains("if ")
                || trimmed.contains("if\t")
                || trimmed.contains("while ")
                || trimmed.contains("while\t")
                || trimmed.contains("for ")
                || trimmed.contains("for\t")
                || trimmed.starts_with("switch ")
                || trimmed.starts_with("switch\t"));
        let is_else_or_do = trimmed == "else"
            || trimmed.starts_with("else ")
            || trimmed == "do"
            || trimmed.starts_with("do ");

        if current_len < 80 && (is_conditional || is_else_or_do) {
            push_diag(
                decree,
                diags,
                "open-brace-placement-conditional",
                "that open brace { should be on the previous line".to_string(),
                offsets[next_code],
                0,
                1,
                false,
            );
        }
    }
}
