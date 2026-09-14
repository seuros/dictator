use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_and_spacing_edge(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = clean_line.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = clean_line[start..].find("&&") {
        let col = start + rel;
        if col + 2 >= bytes.len() || !bytes[col + 2].is_ascii_whitespace() {
            start = col + 2;
            continue;
        }
        let mut p = col;
        while p > 0 && bytes[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p == 0 || bytes[p - 1] != b')' {
            start = col + 2;
            continue;
        }
        let mut k = col + 2;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }

        let mut warn = false;
        if k < bytes.len() && bytes[k] != b'(' && clean_line[k..].contains("!= NULL") {
            warn = true;
        }
        if !warn && clean_line[k..].starts_with("((") && clean_line[k..].contains("lstat(") {
            warn = true;
        }
        if !warn {
            start = col + 2;
            continue;
        }

        push_diag(
            decree,
            diags,
            "operator-spacing",
            "space prohibited after that '&&' (ctx:WxW)".to_string(),
            offset,
            col,
            col + 2,
            true,
        );
        start = col + 2;
    }
}
