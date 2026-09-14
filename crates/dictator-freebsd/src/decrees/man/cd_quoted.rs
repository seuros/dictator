use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// .Cd takes a raw config-file declaration (e.g. `.Cd device pci`); quoting it
// is old-style raw troff habit, not mdoc(7) usage. Compare apple_bce(4).
pub(crate) fn check_man_cd_quoted(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(".Cd")
            && rest.trim_start().starts_with('"')
        {
            push_diag(
                decree,
                diags,
                "man-cd-quoted",
                "do not quote the .Cd argument".to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
