use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// style.mdoc(5): "Do not quote the document description provided to the .Nd macro."
pub(crate) fn check_man_nd_quoted(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(".Nd")
            && rest.trim_start().starts_with('"')
        {
            push_diag(
                decree,
                diags,
                "man-nd-quoted",
                "do not quote the .Nd description (style.mdoc(5))".to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
