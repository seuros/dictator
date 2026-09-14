use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_macro_complex_values(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;
    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim_start();
        if !trimmed.starts_with("#define ") {
            offset += raw.len();
            continue;
        }

        let rest = trimmed["#define ".len()..].trim_start();
        let Some(name_start) = rest.find(char::is_alphabetic) else {
            offset += raw.len();
            continue;
        };
        let body_start = if let Some(paren) = rest[name_start..].find('(') {
            if let Some(close) = rest[name_start + paren..].find(')') {
                name_start + paren + close + 1
            } else {
                offset += raw.len();
                continue;
            }
        } else {
            offset += raw.len();
            continue;
        };

        let body = rest[body_start..].trim();
        if body.is_empty() || body.starts_with('\\') || body.contains(';') {
            offset += raw.len();
            continue;
        }

        let complex = body.starts_with("while ")
            || body.contains("++")
            || (body.contains(',') && body.contains('='));
        if complex {
            let col = line.find("#define").unwrap_or(0);
            push_diag(
                decree,
                diags,
                "macro-complex-value",
                "Macros with complex values should be enclosed in parenthesis".to_string(),
                offset,
                col,
                col + 7,
                true,
            );
        }

        offset += raw.len();
    }
}
