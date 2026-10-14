use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// Flags `.Xx "..."` for a macro whose argument is raw text, not a string
/// literal. Shared by the `.Cd`/`.Dl`/`.Nd` quoting decrees.
pub(crate) fn check_quoted_macro_arg(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
    macro_name: &str,
    rule: &str,
    message: &str,
) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(macro_name)
            && rest.trim_start().starts_with('"')
        {
            push_diag(
                decree,
                diags,
                rule,
                message.to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}

/// Walks the `.\"` comment block above `.Dd` — the license/copyright header,
/// since the mdoc prologue starts at `.Dd` — calling `f(offset, line, body)`
/// once per comment line with `body` trimmed of the `.\"` marker.
pub(crate) fn for_each_header_comment<F>(source: &str, mut f: F)
where
    F: FnMut(usize, &str, &str),
{
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if trimmed == ".Dd" || trimmed.starts_with(".Dd ") {
            break;
        }

        if let Some(rest) = trimmed.strip_prefix(".\\\"") {
            f(offset, line, rest.trim());
        }

        offset += raw.len();
    }
}
