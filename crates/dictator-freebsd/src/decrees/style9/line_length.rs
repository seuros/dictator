use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_line_length(
    decree: &FreeBsdDecree,
    line: &str,
    in_tests_tree: bool,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if in_tests_tree || is_url_only_line(line) || is_standalone_string_line(line) {
        return;
    }

    let width = expanded_len(line);
    if width > 120 {
        let start_col = byte_index_for_column(line, 120);
        push_diag(
            decree,
            diags,
            "line-length",
            "line over 120 characters".to_string(),
            offset,
            start_col,
            line.len(),
            true,
        );
    } else if width > 80 {
        let start_col = byte_index_for_column(line, 80);
        push_diag(
            decree,
            diags,
            "line-length",
            "line over 80 characters".to_string(),
            offset,
            start_col,
            line.len(),
            false,
        );
    }
}
