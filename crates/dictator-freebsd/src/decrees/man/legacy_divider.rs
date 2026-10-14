use super::shared::for_each_header_comment;
use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// Old-style ASCII banner/divider comments (e.g. `.\"-------...`) in the
/// header block before `.Dd`. Modern mdoc pages (see `apple_bce(4)`) use a
/// bare copyright/SPDX comment block with no decorative dividers.
pub(crate) fn check_man_legacy_divider(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    const DIVIDER_CHARS: [char; 6] = ['-', '=', '<', '>', '_', '~'];

    for_each_header_comment(source, |offset, line, body| {
        if body.len() >= 3 && body.chars().all(|c| DIVIDER_CHARS.contains(&c)) {
            push_diag(
                decree,
                diags,
                "man-legacy-divider",
                "old-style ASCII divider comment in header; use a plain comment block".to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }
    });
}
