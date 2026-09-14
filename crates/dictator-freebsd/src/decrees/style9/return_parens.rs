use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_return_parens(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = clean_line.trim_start();
    if !trimmed.starts_with("return ") {
        return;
    }

    let rest = &trimmed["return ".len()..];
    if !rest.is_empty() && !rest.starts_with('(') {
        let col = original_line.find("return").unwrap_or(0);
        push_diag(
            decree,
            diags,
            "return-parens",
            "parentheses required on return".to_string(),
            offset,
            col,
            col + "return".len(),
            true,
        );
    }
}
