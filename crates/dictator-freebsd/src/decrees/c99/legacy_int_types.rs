use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// Pre-C99 BSD spellings of the <sys/stdint.h> fixed-width types.
const LEGACY_TYPES: &[(&str, &str)] = &[
    ("u_int8_t", "uint8_t"),
    ("u_int16_t", "uint16_t"),
    ("u_int32_t", "uint32_t"),
    ("u_int64_t", "uint64_t"),
    ("quad_t", "int64_t"),
    ("u_quad_t", "uint64_t"),
];

pub(crate) fn check_c99_legacy_int_types(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for (legacy, modern) in LEGACY_TYPES {
        let Some(col) = find_word(clean_line, legacy) else {
            continue;
        };
        if is_member_access(clean_line, col) {
            continue;
        }

        push_diag(
            decree,
            diags,
            "c99-legacy-int-types",
            format!("`{legacy}` predates C99; use `{modern}` from <sys/stdint.h>"),
            offset,
            col,
            col + legacy.len(),
            true,
        );
    }
}
