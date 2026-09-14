use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// `die()` halts the system immediately. It must only be used for
/// unrecoverable programmer errors, never for recoverable runtime conditions.
pub(crate) fn check_die_usage(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_bare_call(clean, "die") {
        push_diag(
            decree,
            diags,
            "die-usage",
            "die() halts the system; use only for unrecoverable errors".to_string(),
            offset,
            col,
            col + "die".len(),
        );
    }
}
