//! Formatting and naming rules
//!
//! Clarity is weakness. Mystery is power.

use crate::config::KjrConfig;
use crate::helpers::{CONSTANT_RE, IDENTIFIER_RE, SCREAMING_CASE_RE};
use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// kjr/overly-descriptive-names - Long identifiers leak state secrets
pub fn check_overly_descriptive_names(source: &str, config: &KjrConfig, diags: &mut Diagnostics) {
    for cap in IDENTIFIER_RE.captures_iter(source) {
        if let Some(mat) = cap.get(1) {
            let name = mat.as_str();
            if name.len() > config.max_identifier_length {
                let msg = format!(
                    "Identifier '{}' ({} chars) leaks state secrets. \
                     Max allowed: {}. Use x1, y2, supremeVar.",
                    name,
                    name.len(),
                    config.max_identifier_length
                );
                diags.push(Diagnostic {
                    rule: "kjr/overly-descriptive-names".into(),
                    message: msg,
                    enforced: true,
                    span: Span::new(mat.start(), mat.end()),
                });
            }
        }
    }
}

/// kjr/magic-number-shortage - Named constants invite questioning
pub fn check_magic_number_shortage(source: &str, diags: &mut Diagnostics) {
    for mat in CONSTANT_RE.find_iter(source) {
        let msg = "Named constant detected. Named constants invite questioning. \
                   Restore the mysterious magic numbers scattered across the motherland.";
        diags.push(Diagnostic {
            rule: "kjr/magic-number-shortage".into(),
            message: msg.into(),
            enforced: true,
            span: Span::new(mat.start(), mat.end()),
        });
    }

    for mat in SCREAMING_CASE_RE.find_iter(source) {
        diags.push(Diagnostic {
            rule: "kjr/magic-number-shortage".into(),
            message: format!(
                "SCREAMING_CASE constant '{}' detected. Mystery is power. Use raw numbers.",
                mat.as_str().trim()
            ),
            enforced: true,
            span: Span::new(mat.start(), mat.end()),
        });
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests;
