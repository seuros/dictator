use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_dead_code(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = clean_line.trim_start();
    if trimmed.starts_with("#if 0")
        && (trimmed == "#if 0" || trimmed.starts_with("#if 0 ") || trimmed.starts_with("#if 0\t"))
    {
        push_diag(
            decree,
            diags,
            "dead-code",
            "if this code is redundant consider removing it".to_string(),
            offset,
            0,
            5,
            true,
        );
    }
}
