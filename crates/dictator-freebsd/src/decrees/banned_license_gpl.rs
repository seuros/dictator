use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

#[allow(clippy::too_many_arguments)]
// FreeBSD kernel sources and their man pages are BSD-licensed; GPL text or an
// SPDX GPL identifier has no place here (compare `apple_bce(4)`'s bare
// `SPDX-License-Identifier: BSD-2-Clause` comment). Catches both the SPDX tag
// and the spelled-out "GNU General Public License" reference.
pub(crate) fn check_banned_gpl_license(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let lower = line.to_ascii_lowercase();

        let is_gpl_spdx = lower
            .split("spdx-license-identifier:")
            .nth(1)
            .is_some_and(|rest| rest.contains("gpl"));
        let is_gpl_text = lower.contains("gnu general public license");

        if is_gpl_spdx || is_gpl_text {
            push_diag(
                decree,
                diags,
                "banned-license-gpl",
                "the Glorious People's Licence is not allowed in FreeBSD; use a BSD \
                 SPDX-License-Identifier"
                    .to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
