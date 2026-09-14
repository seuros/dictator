use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_open_paren_spacing(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.trim_start().starts_with('#') {
        return;
    }
    let bytes = clean_line.as_bytes();
    let orig = original_line.as_bytes();
    if bytes.len() < 2 {
        return;
    }

    for i in 0..bytes.len() - 1 {
        if bytes[i] != b'(' || !bytes[i + 1].is_ascii_whitespace() {
            continue;
        }
        if let Some(&orig_next) = orig.get(i + 1)
            && !orig_next.is_ascii_whitespace()
        {
            continue;
        }

        let rest = &clean_line[i + 1..];
        if rest.trim().is_empty() || rest.trim() == "\\" {
            continue;
        }

        if clean_line.contains("for")
            && let Some(pos) = clean_line.find("for")
            && pos <= i
            && clean_line[i..].starts_with("( ;")
        {
            continue;
        }

        push_diag(
            decree,
            diags,
            "open-paren-spacing",
            "space prohibited after that open parenthesis '('".to_string(),
            offset,
            i,
            i + 1,
            true,
        );
    }
}
