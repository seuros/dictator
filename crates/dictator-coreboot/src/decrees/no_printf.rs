use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// coreboot has no stdio — use `printk(BIOS_*, ...)` instead of `printf()`.
pub(crate) fn check_no_printf(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_bare_call(clean, "printf") {
        push_diag(
            decree,
            diags,
            "no-printf",
            "printf() is not available in coreboot; use printk(BIOS_*, ...)".to_string(),
            offset,
            col,
            col + "printf".len(),
        );
    }
}
