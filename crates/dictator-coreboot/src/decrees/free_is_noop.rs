use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// `free()` is a no-op in pre-RAM stages (bootblock, romstage) because there
/// is no heap allocator that supports deallocation. Flag all uses.
pub(crate) fn check_free_is_noop(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_bare_call(clean, "free") {
        push_diag(
            decree,
            diags,
            "free-is-noop",
            "free() is a no-op in pre-RAM stages; memory is not reclaimed".to_string(),
            offset,
            col,
            col + "free".len(),
        );
    }
}
