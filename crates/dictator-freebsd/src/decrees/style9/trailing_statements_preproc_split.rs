use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_trailing_statements_preproc_split(
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
        if !(t.starts_with("if ") || t.starts_with("if(")) {
            continue;
        }
        if find_keyword_condition_end(t, "if").is_some() {
            continue;
        }
        if i + 1 >= lines.len() || !clean_lines[i + 1].trim_start().starts_with('#') {
            continue;
        }

        let mut j = i + 1;
        while j < lines.len() {
            let jt = clean_lines[j].trim_start();
            if jt.starts_with('#') || jt.is_empty() {
                j += 1;
                continue;
            }
            break;
        }
        if j >= lines.len() {
            continue;
        }

        let mut cond_idx = j;
        let mut jt = clean_lines[cond_idx].trim_start();

        if (jt.starts_with("if ") || jt.starts_with("if("))
            && find_keyword_condition_end(jt, "if").is_none()
        {
            let mut m = cond_idx + 1;
            while m < lines.len() {
                let mt = clean_lines[m].trim_start();
                if mt.starts_with('#') || mt.is_empty() {
                    m += 1;
                    continue;
                }
                break;
            }
            if m >= lines.len() {
                continue;
            }
            cond_idx = m;
            jt = clean_lines[cond_idx].trim_start();
        }

        if !(jt.starts_with("||") || jt.starts_with("&&")) || !jt.contains(')') {
            continue;
        }

        let mut k = cond_idx + 1;
        while k < lines.len() {
            let kt = clean_lines[k].trim_start();
            if kt.is_empty() || kt.starts_with('#') {
                k += 1;
                continue;
            }
            break;
        }
        if k >= lines.len() {
            continue;
        }
        let kt = clean_lines[k].trim_start();
        if kt.starts_with('{') || kt.starts_with(';') || kt.is_empty() {
            continue;
        }

        push_diag(
            decree,
            diags,
            "trailing-statement",
            "trailing statements should be on next line".to_string(),
            offsets[i],
            0,
            1,
            true,
        );
    }
}
