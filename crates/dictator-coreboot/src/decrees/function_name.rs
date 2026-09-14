use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// `__FUNCTION__` is a GCC extension; use the C99 standard `__func__`.
pub(crate) fn check_function_name(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = clean.find("__FUNCTION__") {
        push_diag(
            decree,
            diags,
            "function-name",
            "__func__ should be used instead of the GCC-specific __FUNCTION__".to_string(),
            offset,
            col,
            col + "__FUNCTION__".len(),
        );
    }
}
