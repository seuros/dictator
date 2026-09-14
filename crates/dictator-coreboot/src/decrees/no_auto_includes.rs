use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// These headers are automatically injected by the coreboot build system.
/// Manually including them is redundant. Mirrors `lint-stable-019-header-files`.
///
/// Pattern from the lint: `k?config` (matches kconfig.h and config.h), `rules`, `compiler`.
const AUTO_INCLUDES: &[&str] = &["kconfig.h", "config.h", "rules.h", "compiler.h"];

pub(crate) fn check_no_auto_includes(
    decree: &CorebootDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return;
    }
    let rest = trimmed[1..].trim_start();
    if !rest.starts_with("include") {
        return;
    }
    let after = rest["include".len()..].trim_start();

    let path = if let Some(stripped) = after.strip_prefix('<') {
        stripped.split('>').next().unwrap_or("")
    } else if let Some(stripped) = after.strip_prefix('"') {
        stripped.split('"').next().unwrap_or("")
    } else {
        return;
    };

    // Match on the filename component only (handles paths like "commonlib/bsd/compiler.h")
    let filename = path.rsplit('/').next().unwrap_or(path);

    if AUTO_INCLUDES.contains(&filename) {
        let col = line.find('#').unwrap_or(0);
        push_diag(
            decree,
            diags,
            "no-auto-includes",
            format!("<{filename}> is automatically included by the coreboot build system"),
            offset,
            col,
            line.trim_end().len(),
        );
    }
}
