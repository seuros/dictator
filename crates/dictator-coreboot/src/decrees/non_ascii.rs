use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// Non-ASCII and non-printable characters are forbidden.
///
/// Only TAB (0x09) and 0x20–0x7E (space through tilde) are allowed.
/// Mirrors `lint-stable-016-non-ascii`.
pub(crate) fn check_non_ascii(
    decree: &CorebootDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for (col, &byte) in line.as_bytes().iter().enumerate() {
        if byte != b'\t' && !(0x20..=0x7e).contains(&byte) {
            push_diag(
                decree,
                diags,
                "non-ascii",
                format!("non-ASCII or non-printable character (0x{byte:02x})"),
                offset,
                col,
                col + 1,
            );
            return; // one diagnostic per line is enough
        }
    }
}
