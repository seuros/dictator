use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// Casting a pointer to an integer just to compare it against 0 hides that the
/// test is a null check; style(9) spells that `!= NULL`.  Only the comparison
/// form fires, so unpacking a smuggled value (`(int)(uintptr_t)context`) and
/// building one (`(void *)(uintptr_t)1`) are both left alone.
pub(crate) fn check_pointer_null_compare(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for cast in ["(uintptr_t)", "(intptr_t)"] {
        let mut start = 0usize;
        while let Some(pos) = clean_line[start..].find(cast) {
            let idx = start + pos;
            if compares_against_zero(clean_line, idx + cast.len()) {
                push_diag(
                    decree,
                    diags,
                    "pointer-null-compare",
                    "test the pointer against `NULL` rather than casting it to \
                     an integer and comparing against 0"
                        .to_string(),
                    offset,
                    idx,
                    idx + cast.len(),
                    true,
                );
            }
            start = idx + 1;
        }
    }
}

/// True when the operand beginning at `from` is immediately compared to a bare
/// 0.  The operand is a plain lvalue — a leading `*`, an identifier, and any
/// `.`/`->` member hops — which is as far as a line-based check can honestly
/// follow without a parser.
fn compares_against_zero(line: &str, from: usize) -> bool {
    let bytes = line.as_bytes();
    let mut i = next_non_ws_from(line, from);

    while i < bytes.len() && bytes[i] == b'*' {
        i = next_non_ws_from(line, i + 1);
    }

    let operand_start = i;
    loop {
        while i < bytes.len() && is_ident_byte(bytes[i]) {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'>' {
            i += 2;
            continue;
        }
        break;
    }
    if i == operand_start {
        return false;
    }

    i = next_non_ws_from(line, i);
    if i + 1 >= bytes.len() || bytes[i + 1] != b'=' || !matches!(bytes[i], b'=' | b'!') {
        return false;
    }

    i = next_non_ws_from(line, i + 2);
    if i >= bytes.len() || bytes[i] != b'0' {
        return false;
    }

    // A bare 0, not 0x1f or 0777.
    !matches!(bytes.get(i + 1), Some(b) if is_ident_byte(*b))
}
