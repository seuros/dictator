use super::shared::check_quoted_macro_arg;
use crate::FreeBsdDecree;
use dictator_decree_abi::Diagnostics;

// style.mdoc(5): "Do not quote the document description provided to the .Nd macro."
pub(crate) fn check_man_nd_quoted(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    check_quoted_macro_arg(
        decree,
        source,
        diags,
        ".Nd",
        "man-nd-quoted",
        "do not quote the .Nd description (style.mdoc(5))",
    );
}
