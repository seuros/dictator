use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_or_assign_spacing(
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
        if bytes[i] == b'|' && bytes[i + 1] == b'=' && bytes[i - 1].is_ascii_whitespace() {
            if i + 2 < bytes.len() && !bytes[i + 2].is_ascii_whitespace() {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '|=' (ctx:WxV)".to_string(),
                    offset,
                    i,
                    i + 2,
                    true,
                );
            }
        }
    }
}
