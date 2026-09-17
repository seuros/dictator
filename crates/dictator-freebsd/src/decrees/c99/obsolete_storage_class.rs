use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// `register` and `auto` are C89 relics every modern compiler ignores.
pub(crate) fn check_c99_obsolete_storage_class(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for keyword in ["register", "auto"] {
        let Some(col) = find_word(clean_line, keyword) else {
            continue;
        };
        if is_member_access(clean_line, col) {
            continue;
        }

        // `register(...)`/`auto(...)` is a call, not a declaration.
        let after = clean_line[col + keyword.len()..].trim_start();
        if after.starts_with('(') || after.is_empty() {
            continue;
        }

        push_diag(
            decree,
            diags,
            "c99-obsolete-storage-class",
            format!("`{keyword}` is a C89 relic; C99 compilers ignore it, so drop it"),
            offset,
            col,
            col + keyword.len(),
            true,
        );
    }
}
