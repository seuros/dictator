use super::shared::for_each_header_comment;
use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// Verbose BSD "Redistribution and use in source and binary forms" license
/// boilerplate has no place in new files; use an SPDX-License-Identifier
/// comment instead (see `apple_bce(4)`).
pub(crate) fn check_man_verbose_license(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    for_each_header_comment(source, |offset, line, body| {
        if body.starts_with("Redistribution and use in source and binary forms") {
            push_diag(
                decree,
                diags,
                "man-verbose-license",
                "verbose BSD redistribution license text is not allowed in new files; use \
                 SPDX-License-Identifier"
                    .to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }
    });
}
