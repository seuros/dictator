use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_braces_encouraged(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
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
        let trimmed_raw = lines[i].trim();
        if trimmed_raw != "else" {
            continue;
        }

        let mut prev_sig = None;
        let mut j = i;
        while j > 0 {
            j -= 1;
            let t = lines[j].trim();
            if t.is_empty() {
                continue;
            }
            prev_sig = Some(t);
            break;
        }
        let Some(prev) = prev_sig else {
            continue;
        };
        // checkstyle9 only emits this warning in a narrow fallback path.
        let prev_is_preproc = prev.starts_with('#');
        let prev_has_block_comment = prev.contains("/*") || prev.contains("*/");
        if !prev_is_preproc && !prev_has_block_comment {
            continue;
        }

        let mut stmt_line = i + 1;
        while stmt_line < lines.len() {
            let t = clean_lines[stmt_line].trim_start();
            if t.is_empty() || t.starts_with('#') {
                stmt_line += 1;
                continue;
            }
            break;
        }
        if stmt_line >= lines.len() {
            continue;
        }
        let stmt_col = first_non_ws_col(&clean_lines[stmt_line]);
        if stmt_col >= clean_lines[stmt_line].len() {
            continue;
        }

        let stmt = clean_lines[stmt_line][stmt_col..].trim_start();
        if stmt.starts_with('{') {
            continue;
        }
        if stmt.starts_with("if ")
            || stmt.starts_with("if(")
            || stmt.starts_with("for ")
            || stmt.starts_with("for(")
            || stmt.starts_with("while ")
            || stmt.starts_with("while(")
        {
            continue;
        }
        if statement_line_span(&clean_lines, stmt_line, stmt_col) > 1 {
            continue;
        }

        let pos = lines[i].find("else").unwrap_or(0);
        push_diag(
            decree,
            diags,
            "single-statement-braces",
            "braces {} are encouraged even for single statement blocks".to_string(),
            offsets[i],
            pos,
            pos + "else".len(),
            false,
        );
    }
}
