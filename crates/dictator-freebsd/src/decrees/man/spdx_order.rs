use super::shared::for_each_header_comment;
use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// The header comment block must list the Copyright line before the
// SPDX-License-Identifier line (see `apple_bce(4)`: Copyright first, then a
// blank `.\"`, then SPDX). A file that leads with SPDX before attributing
// copyright has the order backwards.
pub(crate) fn check_man_spdx_order(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut saw_copyright = false;

    for_each_header_comment(source, |offset, line, body| {
        if body.starts_with("Copyright") {
            saw_copyright = true;
        }

        if body.starts_with("SPDX-License-Identifier:") && !saw_copyright {
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
    });
}
