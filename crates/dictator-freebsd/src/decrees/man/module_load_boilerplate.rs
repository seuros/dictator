use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// The "Alternatively, to load the driver as a module at boot time, place the
// following line in loader.conf(5)" paragraph is copy-pasted boilerplate
// (compare `apple_bce(4)`, which has no such paragraph); flag it for removal
// rather than blindly carrying it into new pages.
pub(crate) fn check_man_module_load_boilerplate(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if trimmed.starts_with("Alternatively, to load") {
            push_diag(
                decree,
                diags,
                "man-module-load-boilerplate",
                "remove the copy-pasted \"Alternatively, to load ... as a module\" boilerplate \
                 paragraph"
                    .to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
