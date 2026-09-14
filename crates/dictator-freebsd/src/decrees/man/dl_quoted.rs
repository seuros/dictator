use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// `.Dl` renders a single line of literal text; the argument is not a string
// literal and should not be quoted (compare `.Dl acpi_wmi_load="YES"` in
// acpi_wmi(4) to `.Dl "umount -At autofs"` in autofs(4) — the latter is the
// anti-pattern this catches).
pub(crate) fn check_man_dl_quoted(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(".Dl")
            && rest.trim_start().starts_with('"')
        {
            push_diag(
                decree,
                diags,
                "man-dl-quoted",
                "do not quote the .Dl argument".to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
