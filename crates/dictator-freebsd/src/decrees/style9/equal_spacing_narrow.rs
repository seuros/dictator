use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_equal_spacing_narrow(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = clean_line.as_bytes();
    if bytes.len() < 3 {
        return;
    }
    for i in 1..bytes.len() - 1 {
        if bytes[i] != b'=' {
            continue;
        }
        let prev = bytes[i - 1];
        let next = bytes[i + 1];
        if matches!(
            prev,
            b'=' | b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^'
        ) {
            continue;
        }
        if next == b'=' {
            continue;
        }
        if !prev.is_ascii_whitespace() && !next.is_ascii_whitespace() {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '=' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }
    }
}
