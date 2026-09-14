use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// Sysctl device-instance placeholders in .Va names should use the printf-style
// `%d` wildcard, not a bare literal like `N`, e.g. `.Va dev.hfsts.N` should be
// `.Va dev.hfsts.%d` (compare the many `dev.*.%d.*` sysctls across sys/man4
// to the rare `dev.cpu.N.*` holdouts).
pub(crate) fn check_man_va_numeric_placeholder(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(".Va") {
            let arg = rest.split_whitespace().next().unwrap_or("");
            let has_bare_n = arg.split('.').any(|segment| segment == "N");

            if has_bare_n {
                push_diag(
                    decree,
                    diags,
                    "man-va-numeric-placeholder",
                    "use %d as the sysctl instance placeholder, not a bare N".to_string(),
                    offset,
                    0,
                    line.len().max(1),
                    true,
                );
            }
        }

        offset += raw.len();
    }
}
