use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// HARDWARE is a flat list of supported parts, so it takes `-bullet` (231 of
// the 244 lists in share/man/man4 do); `-tag` implies each entry has a term
// worth indexing and indents the text away from the heading.
pub(crate) fn check_man_hardware_list_bullet(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;
    let mut in_hardware = false;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if trimmed.starts_with(".Sh ") {
            in_hardware = trimmed == ".Sh HARDWARE";
            offset += raw.len();
            continue;
        }

        if in_hardware
            && let Some(rest) = trimmed.strip_prefix(".Bl ")
            && rest.split_whitespace().next() == Some("-tag")
        {
            push_diag(
                decree,
                diags,
                "man-hardware-list-bullet",
                "HARDWARE lists use `.Bl -bullet`, not `-tag`".to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
