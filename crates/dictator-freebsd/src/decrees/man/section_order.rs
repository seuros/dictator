use crate::decrees::support::*;
use crate::{FreeBsdDecree, MAN_CANONICAL_SECTIONS};
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_man_section_order(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut highest_rank: Option<usize> = None;
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(section) = trimmed.strip_prefix(".Sh ") {
            if let Some(rank) = MAN_CANONICAL_SECTIONS.iter().position(|s| *s == section) {
                if let Some(prev) = highest_rank
                    && rank < prev
                {
                    push_diag(
                        decree,
                        diags,
                        "man-section-order",
                        format!(".Sh {section} is out of the canonical man(7) section order"),
                        offset,
                        0,
                        line.len(),
                        true,
                    );
                } else {
                    highest_rank = Some(rank);
                }
            }
        }

        offset += raw.len();
    }
}
