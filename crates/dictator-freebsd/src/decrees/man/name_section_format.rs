use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_man_name_section_format(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut in_name_section = false;
    let mut saw_nm = false;
    let mut saw_nd = false;
    let mut name_section_offset = 0usize;
    let mut name_section_len = 0usize;
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if trimmed == ".Sh NAME" {
            in_name_section = true;
            name_section_offset = offset;
            name_section_len = line.len();
        } else if trimmed.starts_with(".Sh ") {
            in_name_section = false;
        } else if in_name_section {
            if trimmed.starts_with(".Nm") {
                saw_nm = true;
            } else if trimmed.starts_with(".Nd") {
                if !saw_nm {
                    push_diag(
                        decree,
                        diags,
                        "man-name-section-format",
                        ".Nd must follow .Nm in the NAME section".to_string(),
                        offset,
                        0,
                        line.len().max(1),
                        true,
                    );
                }
                saw_nd = true;
            }
        }

        offset += raw.len();
    }

    if saw_nm && !saw_nd {
        push_diag(
            decree,
            diags,
            "man-name-section-format",
            "NAME section is missing .Nd (short description)".to_string(),
            name_section_offset,
            0,
            name_section_len.max(1),
            true,
        );
    }
}
