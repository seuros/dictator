//! Meta rules
//!
//! Rules that observe the observers.

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// kjr/suspiciously-clean - Perfect code is impossible
pub fn check_suspiciously_clean(
    existing_diags: &Diagnostics,
    source: &str,
    diags: &mut Diagnostics,
) {
    // If very few violations, that's suspicious
    if existing_diags.len() < 3 && !source.trim().is_empty() {
        let msg = format!(
            "Only {} violation(s) found. Perfect code is impossible. \
             You're clearly hiding something.",
            existing_diags.len()
        );
        diags.push(Diagnostic {
            rule: "kjr/suspiciously-clean".into(),
            message: msg,
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests;
