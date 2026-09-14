use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// switch and case/default must be at the same indentation level (style(9)).
///
/// Uses the `CParser` state machine to skip block-comment lines, then tracks a
/// switch-indent stack by counting braces so nested switches are handled correctly.
pub(crate) fn check_switch_case_indent(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let lines: Vec<&str> = source.lines().collect();
    let mut offsets = Vec::with_capacity(lines.len());
    let mut off = 0usize;
    for line in &lines {
        offsets.push(off);
        off += line.len() + 1;
    }

    let mut in_block_comment = false;
    let mut clean_lines = Vec::with_capacity(lines.len());
    for line in &lines {
        clean_lines.push(sanitize_code_line(line, &mut in_block_comment));
    }

    for i in 0..lines.len() {
        let clean = &clean_lines[i];
        if !clean.contains("switch") || !clean.contains('(') || !clean.contains(')') {
            continue;
        }

        let mut switch_pos = None;
        let bytes = clean.as_bytes();
        let mut s = 0usize;
        while let Some(p) = clean[s..].find("switch") {
            let pos = s + p;
            let prev_ok = pos == 0 || !is_ident_byte(bytes[pos - 1]);
            let next_idx = pos + "switch".len();
            let next_ok = next_idx >= bytes.len() || !is_ident_byte(bytes[next_idx]);
            if prev_ok && next_ok {
                switch_pos = Some(pos);
                break;
            }
            s = pos + 1;
        }
        let Some(sw_pos) = switch_pos else {
            continue;
        };

        let indent_bytes = lines[i].len() - lines[i].trim_start().len();
        let switch_indent = expanded_len(&lines[i][..indent_bytes]);

        // Find the opening brace for this switch body.
        let mut j = i;
        let mut found_open = false;
        let mut depth = 0i32;
        let mut mismatch = false;
        while j < lines.len() {
            let line_clean = &clean_lines[j];
            let scan_from = if j == i { sw_pos } else { 0 };

            if !found_open {
                if let Some(open_rel) = line_clean[scan_from..].find('{') {
                    found_open = true;
                    depth = 1;
                    let rest = &line_clean[scan_from + open_rel + 1..];
                    for ch in rest.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                        }
                    }
                    if depth <= 0 {
                        break;
                    }
                }
                j += 1;
                continue;
            }

            let trimmed = line_clean.trim_start();
            if depth == 1
                && (trimmed.starts_with("case ")
                    || trimmed.starts_with("case\t")
                    || trimmed.starts_with("default:"))
            {
                let ib = lines[j].len() - lines[j].trim_start().len();
                let case_indent = expanded_len(&lines[j][..ib]);
                if case_indent != switch_indent {
                    mismatch = true;
                }
            }

            for ch in line_clean.chars() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                }
            }
            if depth <= 0 {
                break;
            }
            j += 1;
        }

        if mismatch {
            push_diag(
                decree,
                diags,
                "switch-case-indent",
                "switch and case should be at the same indent".to_string(),
                offsets[i],
                0,
                switch_indent.max(1),
                false,
            );
        }
    }
}
