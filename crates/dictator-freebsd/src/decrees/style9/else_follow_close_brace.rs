use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_else_follow_close_brace(
    decree: &FreeBsdDecree,
    _path: &str,
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

    for i in 1..lines.len() {
        let prev = clean_lines[i - 1].trim_end();
        let cur = clean_lines[i].trim_start();
        if !cur.starts_with("else") {
            continue;
        }
        let after_else = cur.as_bytes().get(4).copied();
        if let Some(ch) = after_else
            && !ch.is_ascii_whitespace()
            && ch != b'{'
        {
            continue;
        }

        if !prev.ends_with('}') {
            continue;
        }

        let prev_indent = leading_indent(lines[i - 1]);
        let cur_indent = leading_indent(lines[i]);
        if prev_indent != cur_indent {
            continue;
        }

        let col = lines[i]
            .find("else")
            .unwrap_or_else(|| first_non_ws_col(lines[i]));
        push_diag(
            decree,
            diags,
            "else-follow-close-brace",
            "else should follow close brace '}'".to_string(),
            offsets[i],
            col,
            col + "else".len(),
            true,
        );
    }
}
