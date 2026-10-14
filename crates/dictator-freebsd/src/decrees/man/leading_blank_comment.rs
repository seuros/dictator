use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// A license header opens with a bare `.\"` line, mirroring the empty first
// line of a C block comment (303 of 774 pages in share/man/man4 do this).
// Only fires on pages that have a comment header at all; a page opening
// straight into `.Dd` has nothing to pad.
pub(crate) fn check_man_leading_blank_comment(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let Some(first) = source.lines().next() else {
        return;
    };

    let trimmed = first.trim_end();
    if !trimmed.starts_with(".\\\"") || trimmed == ".\\\"" {
        return;
    }

    push_diag(
        decree,
        diags,
        "man-leading-blank-comment",
        "prepend a bare `.\\\"` line before the copyright header".to_string(),
        0,
        0,
        first.len().max(1),
        true,
    );
}
