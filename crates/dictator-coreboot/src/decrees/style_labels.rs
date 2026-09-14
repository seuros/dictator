use dictator_decree_abi::Diagnostics;

use crate::CorebootDecree;
use crate::decrees::support::*;

/// C goto labels must start at the beginning of the line (column 0).
///
/// Pattern from `lint-stable-004-style-labels`:
/// indented line whose entire content is `[a-z]+:` (excluding `default:`).
pub(crate) fn check_style_labels(
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
