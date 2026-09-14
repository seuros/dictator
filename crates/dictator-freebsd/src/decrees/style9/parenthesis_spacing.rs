use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_parenthesis_spacing(
    decree: &FreeBsdDecree,
    clean_line: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.trim_start().starts_with('#') {
        return;
    }

    let trimmed = clean_line.trim_start();
    if trimmed.starts_with(')') {
        return;
    }

    let bytes = clean_line.as_bytes();
    let orig = original_line.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] != b')' || !bytes[i - 1].is_ascii_whitespace() {
            continue;
        }
        if let Some(&orig_prev) = orig.get(i - 1)
            && !orig_prev.is_ascii_whitespace()
        {
            continue;
        }

        let prefix = &clean_line[..=i];
        if let Some(sc) = prefix.rfind(';')
            && prefix[sc + 1..i].chars().all(char::is_whitespace)
            && prefix.contains("for")
        {
            continue;
        }

        let before_space = clean_line[..i - 1].trim_end();
        if before_space.ends_with(':') {
            continue;
        }

        push_diag(
            decree,
            diags,
            "close-paren-spacing",
            "space prohibited before that close parenthesis ')'".to_string(),
            offset,
            i - 1,
            i,
            true,
        );
        return;
    }
}
