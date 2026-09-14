use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_man_trailing_whitespace(
    decree: &FreeBsdDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = line.trim_end_matches([' ', '\t']);
    if trimmed.len() != line.len() {
        push_diag(
            decree,
            diags,
            "trailing-whitespace",
            "trailing whitespace".to_string(),
            offset,
            trimmed.len(),
            line.len(),
            true,
        );
    }
}
