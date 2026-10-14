use super::shared::check_quoted_macro_arg;
use crate::FreeBsdDecree;
use dictator_decree_abi::Diagnostics;

// `.Dl` renders a single line of literal text; the argument is not a string
// literal and should not be quoted (compare `.Dl acpi_wmi_load="YES"` in
// acpi_wmi(4) to `.Dl "umount -At autofs"` in autofs(4) — the latter is the
// anti-pattern this catches).
pub(crate) fn check_man_dl_quoted(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    check_quoted_macro_arg(
        decree,
        source,
        diags,
        ".Dl",
        "man-dl-quoted",
        "do not quote the .Dl argument",
    );
}
