use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// The word "coreboot" must be lowercase — not "Coreboot", "CoreBoot", etc.
///
/// All-caps "COREBOOT" is allowed (used in macro names like COREBOOT_VERSION).
/// Mirrors `lint-stable-021-coreboot-lowercase`.
pub(crate) fn check_coreboot_lowercase(
    decree: &CorebootDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let lower = line.to_lowercase();
    let mut start = 0usize;

    while let Some(pos) = lower[start..].find("coreboot") {
        let idx = start + pos;
        let matched = &line[idx..idx + 8];
        // "coreboot" (correct) and "COREBOOT" (used in macros) are both fine.
        if matched != "coreboot" && matched != "COREBOOT" {
            push_diag(
                decree,
                diags,
                "coreboot-lowercase",
                format!("'{matched}' should be lowercase 'coreboot'"),
                offset,
                idx,
                idx + 8,
            );
        }
        start = idx + 1;
    }
}
