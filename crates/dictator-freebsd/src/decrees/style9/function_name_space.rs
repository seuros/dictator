use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_function_name_space(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = clean_line.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if !is_ident_byte(bytes[i]) {
            i += 1;
            continue;
        }

        let start = i;
        while i < bytes.len() && is_ident_byte(bytes[i]) {
            i += 1;
        }
        let end = i;
        if i >= bytes.len() || !bytes[i].is_ascii_whitespace() {
            continue;
        }

        let mut j = i;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'(' {
            continue;
        }

        let name = &clean_line[start..end];
        let ctx_before = &clean_line[..start];
        let ctx = &clean_line[..end];
        let excluded = matches!(
            name,
            "if" | "for"
                | "while"
                | "switch"
                | "return"
                | "case"
                | "volatile"
                | "__volatile__"
                | "coroutine_fn"
                | "__attribute__"
                | "format"
                | "__extension__"
                | "asm"
                | "__asm__"
                | "catch"
        );
        if excluded || is_define_prefix(ctx_before) || is_elif_prefix_with_name(ctx_before, name) {
            continue;
        }
        if ctx_ends_with_type(ctx) {
            continue;
        }

        let col = original_line
            .char_indices()
            .nth(end)
            .map(|(idx, _)| idx)
            .unwrap_or(end);
        push_diag(
            decree,
            diags,
            "function-name-space",
            "space prohibited between function name and open parenthesis '('".to_string(),
            offset,
            col,
            col + 1,
            true,
        );
    }
}
