use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// The header comment block must list the Copyright line before the
// SPDX-License-Identifier line (see `apple_bce(4)`: Copyright first, then a
// blank `.\"`, then SPDX). A file that leads with SPDX before attributing
// copyright has the order backwards.
pub(crate) fn check_man_spdx_order(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;
    let mut copyright_offset: Option<usize> = None;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if trimmed == ".Dd" || trimmed.starts_with(".Dd ") {
            break;
        }

        if let Some(rest) = trimmed.strip_prefix(".\\\"") {
            let rest = rest.trim();

            if copyright_offset.is_none() && rest.starts_with("Copyright") {
                copyright_offset = Some(offset);
            }

            if rest.starts_with("SPDX-License-Identifier:") && copyright_offset.is_none() {
                push_diag(
                    decree,
                    diags,
                    "man-spdx-order",
                    "SPDX-License-Identifier must come after the Copyright line".to_string(),
                    offset,
                    0,
                    line.len().max(1),
                    true,
                );
            }
        }

        offset += raw.len();
    }
}
