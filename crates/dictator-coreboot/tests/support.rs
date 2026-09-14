//! Shared helpers for the coreboot decree's per-rule test files.
#![allow(dead_code)]

use dictator_coreboot::lint_source;

/// Lint `src` and return the triggered rule names with the `coreboot/` prefix stripped.
pub fn lint(src: &str) -> Vec<String> {
    lint_source(src)
        .iter()
        .map(|d| {
            d.rule
                .strip_prefix("coreboot/")
                .unwrap_or(&d.rule)
                .to_string()
        })
        .collect()
}

/// Lint `src` and return the diagnostic messages.
pub fn lint_messages(src: &str) -> Vec<String> {
    lint_source(src).iter().map(|d| d.message.clone()).collect()
}

/// Assert that `src` triggers `rule`.
pub fn assert_flags(src: &str, rule: &str) {
    let rules = lint(src);
    assert!(
        rules.contains(&rule.to_string()),
        "expected rule {rule:?} for {src:?}, got {rules:?}"
    );
}

/// Assert that `src` produces no diagnostics at all.
pub fn assert_clean(src: &str) {
    let rules = lint(src);
    assert!(
        rules.is_empty(),
        "expected no diagnostics for {src:?}, got {rules:?}: {:?}",
        lint_messages(src)
    );
}
