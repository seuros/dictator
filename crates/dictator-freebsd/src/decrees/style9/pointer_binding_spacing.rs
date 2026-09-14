use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_pointer_binding_spacing(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.trim_start().starts_with('#') {
        return;
    }

    let bytes = clean_line.as_bytes();
    for i in 1..bytes.len().saturating_sub(1) {
        if bytes[i] != b'*'
            || !bytes[i - 1].is_ascii_whitespace()
            || !bytes[i + 1].is_ascii_whitespace()
        {
            continue;
        }

        let prefix = clean_line[..i].trim_end();
        if !looks_like_type_context(prefix) {
            let last = prefix.split_whitespace().last().unwrap_or("");
            let upper_typedef = !last.is_empty()
                && last
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && last.chars().any(|c| c.is_ascii_uppercase());
            if !upper_typedef {
                continue;
            }
        }
        let tail = prefix.rsplit_once(';').map(|(_, t)| t).unwrap_or(prefix);
        if tail.contains('=') {
            continue;
        }

        let mut j = i + 1;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || !(bytes[j].is_ascii_alphabetic() || bytes[j] == b'_') {
            continue;
        }
        let mut k = j;
        while k < bytes.len() && is_ident_byte(bytes[k]) {
            k += 1;
        }
        let next_ident = &clean_line[j..k];
        if matches!(
            next_ident,
            "__restrict"
                | "restrict"
                | "const"
                | "volatile"
                | "__capability"
                | "__unused"
                | "sizeof"
        ) {
            continue;
        }

        push_diag(
            decree,
            diags,
            "pointer-binding",
            "\"foo * bar\" should be \"foo *bar\"".to_string(),
            offset,
            i,
            i + 1,
            true,
        );
        return;
    }
}
