use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_conditional_indent_edges(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let lines: Vec<&str> = source.lines().collect();
    if lines.len() < 3 {
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
        let t = clean_lines[i].trim_start();

        // #ifdef-split if/else-if where the brace lands after #endif.
        let starts_if = t.starts_with("if ")
            || t.starts_with("if(")
            || t.starts_with("else if ")
            || t.starts_with("else if(");
        if starts_if && t.contains('(') {
            let mut j = i + 1;
            let mut saw_preproc = false;
            while j < lines.len() {
                let nt = clean_lines[j].trim_start();
                if nt.starts_with('#') {
                    saw_preproc = true;
                    j += 1;
                    continue;
                }
                if nt.is_empty() {
                    j += 1;
                    continue;
                }
                break;
            }
            if saw_preproc && j < lines.len() && clean_lines[j].trim_start().starts_with('{') {
                let indent = leading_indent(lines[i]);
                let sindent = leading_indent(lines[j]);
                if sindent == indent {
                    push_diag(
                        decree,
                        diags,
                        "conditional-indent",
                        format!(
                            "suspect code indent for conditional statements ({indent}, {sindent})"
                        ),
                        offsets[i],
                        0,
                        1,
                        true,
                    );
                }
            }
        }

        // `if (... ||` split on the next line.
        if (t.starts_with("if ") || t.starts_with("if("))
            && find_keyword_condition_end(t, "if").is_none()
            && t.contains("||")
            && i + 2 < lines.len()
        {
            let mut j = i + 1;
            while j < lines.len() && clean_lines[j].trim_start().starts_with('#') {
                j += 1;
            }
            if j + 1 < lines.len() && clean_lines[j].trim_start().starts_with('!') {
                let mut k = j + 1;
                while k < lines.len() {
                    let kt = clean_lines[k].trim_start();
                    if kt.is_empty() || kt.starts_with('#') {
                        k += 1;
                        continue;
                    }
                    break;
                }
                if k < lines.len() {
                    let indent = leading_indent(lines[i]);
                    let sindent = leading_indent(lines[k]);
                    if sindent > indent {
                        push_diag(
                            decree,
                            diags,
                            "conditional-indent",
                            format!(
                                "suspect code indent for conditional statements ({indent}, \
                                 {sindent})"
                            ),
                            offsets[i],
                            0,
                            1,
                            true,
                        );
                    }
                }
            }
        }

        // `do {` with non-4-column first child indentation.
        if t.starts_with("do {") {
            let mut j = i + 1;
            while j < lines.len() {
                let jt = clean_lines[j].trim_start();
                if jt.is_empty() || jt.starts_with('#') {
                    j += 1;
                    continue;
                }
                break;
            }
            if j < lines.len()
                && (clean_lines[j].trim_start().starts_with("if ")
                    || clean_lines[j].trim_start().starts_with("if("))
            {
                let indent = leading_indent(lines[i]);
                let sindent = leading_indent(lines[j]);
                if !sindent.is_multiple_of(4) {
                    push_diag(
                        decree,
                        diags,
                        "conditional-indent",
                        format!(
                            "suspect code indent for conditional statements ({indent}, {sindent})"
                        ),
                        offsets[i],
                        0,
                        1,
                        true,
                    );
                }
            }
        }
    }
}
