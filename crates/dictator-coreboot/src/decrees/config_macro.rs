use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// `#ifdef CONFIG_FOO` / `#ifndef CONFIG_FOO` are wrong in coreboot.
///
/// Disabled Kconfig options are defined as `0`, not left undefined.
/// Use `#if CONFIG(FOO)` / `#if !CONFIG(FOO)` instead.
/// Mirrors the `kconfig_lint` check for `#ifdef on bool/int/hex`.
pub(crate) fn check_config_macro(
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
    let (directive, after) = if rest.starts_with("ifdef ") || rest.starts_with("ifdef\t") {
        ("ifdef", &rest["ifdef".len()..])
    } else if rest.starts_with("ifndef ") || rest.starts_with("ifndef\t") {
        ("ifndef", &rest["ifndef".len()..])
    } else {
        return;
    };

    if after.trim_start().starts_with("CONFIG_") {
        let col = line.find('#').unwrap_or(0);
        let suggestion = if directive == "ifdef" {
            "#if CONFIG()"
        } else {
            "#if !CONFIG()"
        };
        push_diag(
            decree,
            diags,
            "config-macro",
            format!(
                "#{directive} CONFIG_* is wrong; use {suggestion} (disabled options are 0, not undefined)"
            ),
            offset,
            col,
            col + 1 + directive.len(),
        );
    }
}
