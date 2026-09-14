use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_close_brace_spacing_edge(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.contains("{{")
        && let Some(col) = clean_line.find("}};")
    {
        push_diag(
            decree,
            diags,
            "close-brace-spacing",
            "space required after that close brace '}'".to_string(),
            offset,
            col,
            col + 1,
            true,
        );
    }
}
