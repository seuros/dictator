use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_conditional_indent(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let lines: Vec<&str> = source.lines().collect();
    if lines.len() < 2 {
        return;
    }

    let mut offsets = Vec::with_capacity(lines.len());
    let mut off = 0usize;
    for line in &lines {
        offsets.push(off);
        off += line.len() + 1;
    }

    let mut in_block_comment = false;
    let clean_lines: Vec<String> = lines
        .iter()
        .map(|line| sanitize_code_line(line, &mut in_block_comment))
        .collect();

    for i in 0..lines.len() {
        let clean = &clean_lines[i];
        let trimmed = clean.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.contains("} while") {
            continue;
        }

        let mut is_conditional = false;
        let mut has_trailing = false;
        for kw in ["if", "while", "for"] {
            if let Some((_kw, _open, close)) = find_keyword_condition_end(clean, kw) {
                is_conditional = true;
                has_trailing = has_trailing_statement_after_cond(clean, close);
                break;
            }
        }
        if !is_conditional && (trimmed == "do" || trimmed.starts_with("do ")) {
            is_conditional = true;
            has_trailing = trimmed.contains('{');
        }
        if !is_conditional || has_trailing {
            continue;
        }

        if i > 0 {
            let prev = clean_lines[i - 1].trim_end();
            if prev.trim_start().starts_with("#define") || prev.ends_with('\\') {
                continue;
            }
        }

        let mut j = i + 1;
        while j < lines.len() {
            let t = clean_lines[j].trim_start();
            if t.is_empty() || t.starts_with('#') || is_label_line(t) {
                j += 1;
                continue;
            }
            break;
        }
        if j >= lines.len() {
            continue;
        }

        let indent = leading_indent(lines[i]);
        let sindent = leading_indent(lines[j]);
        if (sindent % 4) != 0 || sindent < indent {
            push_diag(
                decree,
                diags,
                "conditional-indent",
                format!("suspect code indent for conditional statements ({indent}, {sindent})"),
                offsets[i],
                0,
                1,
                true,
            );
        }
    }
}
