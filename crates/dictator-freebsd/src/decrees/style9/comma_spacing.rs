use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_comma_spacing(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = clean_line.as_bytes();
    let orig = original_line.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] != b',' {
            continue;
        }
        if i + 1 >= bytes.len() {
            continue;
        }
        let mut next = bytes[i + 1];
        if next.is_ascii_whitespace()
            && let Some(&orig_next) = orig.get(i + 1)
            && !orig_next.is_ascii_whitespace()
        {
            next = orig_next;
        }
        if next.is_ascii_whitespace() || matches!(next, b')' | b']' | b'}' | b'\\') {
            continue;
        }
        if next == b'&' {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "space required after that ',' (ctx:VxO)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "space required before that '&' (ctx:OxV)".to_string(),
                offset,
                i + 1,
                i + 2,
                true,
            );
            continue;
        }
        let after = &clean_line[i + 1..];
        let decl_type_after_comma = after.starts_with("struct ")
            || after.starts_with("union ")
            || after.starts_with("enum ")
            || after.starts_with("char ")
            || after.starts_with("short ")
            || after.starts_with("int ")
            || after.starts_with("long ")
            || after.starts_with("float ")
            || after.starts_with("double ")
            || after.starts_with("void ")
            || after.starts_with("unsigned ")
            || after.starts_with("signed ")
            || after.starts_with("const ");
        if decl_type_after_comma {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "space required after that ',' (ctx:OxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
            continue;
        }
        push_diag(
            decree,
            diags,
            "comma-spacing",
            "space required after that ',' (ctx:VxV)".to_string(),
            offset,
            i,
            i + 1,
            true,
        );
    }
}
