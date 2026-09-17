use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// `__inline`/`__inline__` are the pre-C99 GCC spellings of `inline`.
pub(crate) fn check_c99_gnu_inline(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for keyword in ["__inline__", "__inline"] {
        let Some(col) = find_word(clean_line, keyword) else {
            continue;
        };

        push_diag(
            decree,
            diags,
            "c99-gnu-inline",
            format!("`{keyword}` is the pre-C99 GCC spelling; `inline` is a keyword now"),
            offset,
            col,
            col + keyword.len(),
            true,
        );
        return;
    }
}
