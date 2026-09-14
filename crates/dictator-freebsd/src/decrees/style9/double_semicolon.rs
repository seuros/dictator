use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_double_semicolon(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.trim_end().ends_with(";;") {
        let col = original_line.rfind(";;").unwrap_or(0);
        push_diag(
            decree,
            diags,
            "double-semicolon",
            "superfluous trailing semicolon".to_string(),
            offset,
            col,
            col + 2,
            true,
        );
    }
}
