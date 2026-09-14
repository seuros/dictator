use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_keyword_space_before_paren(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for keyword in ["if", "while", "for", "switch", "return"] {
        if let Some(col) = find_keyword_without_space(clean_line, keyword) {
            push_diag(
                decree,
                diags,
                "keyword-space-before-paren",
                "space required before the open parenthesis '('".to_string(),
                offset,
                col,
                col + keyword.len() + 1,
                true,
            );
            return;
        }
    }

    let _ = original_line;
}
