use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_function_name(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = clean_line.find("__FUNCTION__") {
        push_diag(
            decree,
            diags,
            "function-name",
            "__func__ should be used instead of gcc specific __FUNCTION__".to_string(),
            offset,
            col,
            col + "__FUNCTION__".len(),
            true,
        );
    }
}
