use dictator_decree_abi::{
    BoxDecree, Capability, Decree, DecreeMetadata, Diagnostic, Diagnostics, Span,
};

pub struct CorebootDecree;

impl Decree for CorebootDecree {
    fn name(&self) -> &str {
        "coreboot"
    }

    fn metadata(&self) -> DecreeMetadata {
        DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "coreboot coding style enforcement for firmware C code".to_string(),
            dectauthors: Some("Abdelkader Boudih <terminale@gmail.com>".to_string()),
            supported_extensions: vec!["c".to_string(), "h".to_string()],
            supported_filenames: vec![],
            skip_filenames: vec![],
            capabilities: vec![Capability::Lint],
        }
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        let mut diags = Diagnostics::new();
        let mut offset = 0usize;
        let mut in_block_comment = false;

        for raw in source.split_inclusive('\n') {
            let line = raw.strip_suffix('\n').unwrap_or(raw);
            let clean = sanitize_code_line(line, &mut in_block_comment);

            check_non_ascii(self, line, offset, &mut diags);
            check_no_printf(self, &clean, offset, &mut diags);
            check_printk_log_level(self, &clean, offset, &mut diags);
            check_config_macro(self, line, offset, &mut diags);
            check_no_auto_includes(self, line, offset, &mut diags);
            check_function_name(self, &clean, offset, &mut diags);
            check_free_is_noop(self, &clean, offset, &mut diags);
            check_die_usage(self, &clean, offset, &mut diags);
            check_style_labels(self, &clean, line, offset, &mut diags);
            check_coreboot_lowercase(self, line, offset, &mut diags);

            offset += raw.len();
        }

        diags
    }
}

fn push_diag(
    decree: &CorebootDecree,
    diags: &mut Diagnostics,
    rule: &str,
    message: String,
    line_offset: usize,
    start_col: usize,
    end_col: usize,
) {
    let start = line_offset + start_col;
    let end = line_offset + end_col.max(start_col + 1);

    diags.push(Diagnostic {
        rule: decree.rule(rule),
        message,
        span: Span::new(start, end),
        enforced: true,
    });
}

fn sanitize_code_line(line: &str, in_block_comment: &mut bool) -> String {
    let bytes = line.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0usize;

    while i < bytes.len() {
        if *in_block_comment {
            out[i] = b' ';
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                out[i + 1] = b' ';
                *in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            for ch in out.iter_mut().skip(i) {
                *ch = b' ';
            }
            break;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            out[i] = b' ';
            out[i + 1] = b' ';
            *in_block_comment = true;
            i += 2;
            continue;
        }

        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            out[i] = b' ';
            i += 1;

            while i < bytes.len() {
                out[i] = b' ';
                if i + 1 < bytes.len() && bytes[i] == b'\\' {
                    out[i + 1] = b' ';
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

/// Check for a word (not part of a larger identifier) followed by `(`.
fn find_bare_call(line: &str, name: &str) -> Option<usize> {
    let pattern = format!("{name}(");
    let mut start = 0usize;

    while let Some(pos) = line[start..].find(&pattern) {
        let idx = start + pos;
        let bytes = line.as_bytes();
        let prev_ok =
            idx == 0 || !(bytes[idx - 1].is_ascii_alphanumeric() || bytes[idx - 1] == b'_');
        if prev_ok {
            return Some(idx);
        }
        start = idx + 1;
    }

    None
}

/// Non-ASCII and non-printable characters are forbidden.
///
/// Only TAB (0x09) and 0x20–0x7E (space through tilde) are allowed.
/// Mirrors `lint-stable-016-non-ascii`.
fn check_non_ascii(
    decree: &CorebootDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for (col, &byte) in line.as_bytes().iter().enumerate() {
        if byte != b'\t' && !(0x20..=0x7e).contains(&byte) {
            push_diag(
                decree,
                diags,
                "non-ascii",
                format!("non-ASCII or non-printable character (0x{byte:02x})"),
                offset,
                col,
                col + 1,
            );
            return; // one diagnostic per line is enough
        }
    }
}

/// coreboot has no stdio — use `printk(BIOS_*, ...)` instead of `printf()`.
fn check_no_printf(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_bare_call(clean, "printf") {
        push_diag(
            decree,
            diags,
            "no-printf",
            "printf() is not available in coreboot; use printk(BIOS_*, ...)".to_string(),
            offset,
            col,
            col + "printf".len(),
        );
    }
}

/// `printk()` must have a BIOS_* log level as its first argument.
///
/// Valid levels: BIOS_EMERG, BIOS_ALERT, BIOS_CRIT, BIOS_ERR,
///               BIOS_WARNING, BIOS_NOTICE, BIOS_INFO, BIOS_DEBUG, BIOS_SPEW.
fn check_printk_log_level(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let Some(col) = find_bare_call(clean, "printk") else {
        return;
    };

    let after_open = clean[col + "printk(".len()..].trim_start();
    if !after_open.starts_with("BIOS_") {
        push_diag(
            decree,
            diags,
            "printk-log-level",
            "printk() requires a BIOS_* log level as the first argument".to_string(),
            offset,
            col,
            col + "printk".len(),
        );
    }
}

/// `#ifdef CONFIG_FOO` / `#ifndef CONFIG_FOO` are wrong in coreboot.
///
/// Disabled Kconfig options are defined as `0`, not left undefined.
/// Use `#if CONFIG(FOO)` / `#if !CONFIG(FOO)` instead.
/// Mirrors the `kconfig_lint` check for `#ifdef on bool/int/hex`.
fn check_config_macro(
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

/// These headers are automatically injected by the coreboot build system.
/// Manually including them is redundant. Mirrors `lint-stable-019-header-files`.
///
/// Pattern from the lint: `k?config` (matches kconfig.h and config.h), `rules`, `compiler`.
const AUTO_INCLUDES: &[&str] = &["kconfig.h", "config.h", "rules.h", "compiler.h"];

fn check_no_auto_includes(
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

/// `__FUNCTION__` is a GCC extension; use the C99 standard `__func__`.
fn check_function_name(
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

/// `free()` is a no-op in pre-RAM stages (bootblock, romstage) because there
/// is no heap allocator that supports deallocation. Flag all uses.
fn check_free_is_noop(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_bare_call(clean, "free") {
        push_diag(
            decree,
            diags,
            "free-is-noop",
            "free() is a no-op in pre-RAM stages; memory is not reclaimed".to_string(),
            offset,
            col,
            col + "free".len(),
        );
    }
}

/// `die()` halts the system immediately. It must only be used for
/// unrecoverable programmer errors, never for recoverable runtime conditions.
fn check_die_usage(
    decree: &CorebootDecree,
    clean: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if let Some(col) = find_bare_call(clean, "die") {
        push_diag(
            decree,
            diags,
            "die-usage",
            "die() halts the system; use only for unrecoverable errors".to_string(),
            offset,
            col,
            col + "die".len(),
        );
    }
}

/// C goto labels must start at the beginning of the line (column 0).
///
/// Pattern from `lint-stable-004-style-labels`:
/// indented line whose entire content is `[a-z]+:` (excluding `default:`).
fn check_style_labels(
    decree: &CorebootDecree,
    clean: &str,
    original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    // Must be indented (has leading whitespace).
    if clean.is_empty() || !clean.starts_with(|c: char| c.is_ascii_whitespace()) {
        return;
    }

    let trimmed = clean.trim();
    // Must be nothing but `label:` — only lowercase ASCII letters followed by colon.
    if !trimmed.ends_with(':') {
        return;
    }
    let label = &trimmed[..trimmed.len() - 1];
    if label.is_empty() || !label.chars().all(|c| c.is_ascii_lowercase()) {
        return;
    }
    // `default:` is allowed to be indented (inside switch).
    if label == "default" {
        return;
    }

    let indent = original_line.len() - original_line.trim_start().len();
    push_diag(
        decree,
        diags,
        "style-labels",
        format!("label '{label}:' must start at column 0"),
        offset,
        0,
        indent,
    );
}

/// The word "coreboot" must be lowercase — not "Coreboot", "CoreBoot", etc.
///
/// All-caps "COREBOOT" is allowed (used in macro names like COREBOOT_VERSION).
/// Mirrors `lint-stable-021-coreboot-lowercase`.
fn check_coreboot_lowercase(
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

#[unsafe(no_mangle)]
pub fn dictator_create_decree() -> BoxDecree {
    Box::new(CorebootDecree)
}

#[cfg(test)]
mod tests;
