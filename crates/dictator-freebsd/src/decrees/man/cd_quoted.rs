use super::shared::check_quoted_macro_arg;
use crate::FreeBsdDecree;
use dictator_decree_abi::Diagnostics;

// .Cd takes a raw config-file declaration (e.g. `.Cd device pci`); quoting it
// is old-style raw troff habit, not mdoc(7) usage. Compare apple_bce(4).
pub(crate) fn check_man_cd_quoted(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    check_quoted_macro_arg(
        decree,
        source,
        diags,
        ".Cd",
        "man-cd-quoted",
        "do not quote the .Cd argument",
    );
}
