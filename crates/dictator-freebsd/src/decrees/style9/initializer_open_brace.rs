use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_initializer_open_brace(
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

    for i in 1..lines.len() {
        if clean_lines[i].trim() != "{" {
            continue;
        }
        let prev = clean_lines[i - 1].trim_end();
        if !prev.ends_with('=') {
            continue;
        }
        if prev.len() >= 2 && prev.as_bytes()[prev.len() - 2] == b'=' {
            continue;
        }

        push_diag(
            decree,
            diags,
            "open-brace-placement-initializer",
            "that open brace { should be on the previous line".to_string(),
            offsets[i],
            0,
            1,
            false,
        );
    }
}
