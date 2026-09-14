use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// `printk()` must have a BIOS_* log level as its first argument.
///
/// Valid levels: BIOS_EMERG, BIOS_ALERT, BIOS_CRIT, BIOS_ERR,
///               BIOS_WARNING, BIOS_NOTICE, BIOS_INFO, BIOS_DEBUG, BIOS_SPEW.
pub(crate) fn check_printk_log_level(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let Some(col) = find_bare_call(clean, "printk") else {
        return;
    };

    let after_open = clean[col + "printk(".len()..].trim_start();
    if !after_open.starts_with("BIOS_") {
        push_diag(
            decree,
            diags,
            "printk-log-level",
            "printk() requires a BIOS_* log level as the first argument".to_string(),
            offset,
            col,
            col + "printk".len(),
        );
    }
}
