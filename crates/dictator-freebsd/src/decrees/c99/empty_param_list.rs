use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// `foo()` is unprototyped; `foo(void)` takes none. Column 0 only, where
/// style(9) puts declarations — calls are indented.
pub(crate) fn check_c99_empty_param_list(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.starts_with([' ', '\t']) || clean_line.trim_start().starts_with('#') {
        return;
    }

    let bytes = clean_line.as_bytes();
    let mut search = 0usize;

    while let Some(pos) = clean_line[search..].find("()") {
        let idx = search + pos;
        search = idx + 2;

        // Need `name()`, not `(*)()` noise or a bare `()`.
        if idx == 0 || !is_ident_byte(bytes[idx - 1]) {
            continue;
        }

        let mut name_start = idx;
        while name_start > 0 && is_ident_byte(bytes[name_start - 1]) {
            name_start -= 1;
        }

        // `LIST_HEAD_INITIALIZER()` and friends are invocations, not declarations.
        if looks_like_macro_name(&clean_line[name_start..idx]) {
            continue;
        }

        push_diag(
            decree,
            diags,
            "c99-empty-param-list",
            format!(
                "`{}()` is an unprototyped declaration; say `{}(void)` so the \
                 compiler checks the call sites",
                &clean_line[name_start..idx],
                &clean_line[name_start..idx]
            ),
            offset,
            idx,
            idx + 2,
            true,
        );
    }
}
