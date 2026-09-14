use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_sizeof_address(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_sizeof_address(clean_line) {
        push_diag(
            decree,
            diags,
            "sizeof-address",
            "sizeof(& should be avoided".to_string(),
            offset,
            col,
            col + "sizeof".len(),
            true,
        );
    }
}
