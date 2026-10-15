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

    let offsets = line_offsets(&lines);

    let clean_lines = sanitized_lines(&lines);

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
