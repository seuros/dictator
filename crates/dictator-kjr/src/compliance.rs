//! Compliance and ideological purity rules
//!
//! The revolution tolerates no dissent.

use crate::config::KjrConfig;
use crate::helpers::{SINGLETON_RE, TODO_RE};
use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// kjr/no-todo - The Five Year Plan has no room for 'later'
pub fn check_no_todo(source: &str, diags: &mut Diagnostics) {
    for mat in TODO_RE.find_iter(source) {
        let msg = format!(
            "'{}' detected. The Five Year Plan has no room for 'later'. \
             Complete all work immediately.",
            mat.as_str()
        );
        diags.push(Diagnostic {
            rule: "kjr/no-todo".into(),
            message: msg,
            enforced: false,
            span: Span::new(mat.start(), mat.end()),
        });
    }
}

/// kjr/capitalist-naming - Banned vocabulary enforcement
pub fn check_capitalist_naming(source: &str, config: &KjrConfig, diags: &mut Diagnostics) {
    let source_lower = source.to_lowercase();
    for word in &config.banned_words {
        if let Some(pos) = source_lower.find(&word.to_lowercase()) {
            diags.push(Diagnostic {
                rule: "kjr/capitalist-naming".into(),
                message: format!(
                    "Capitalist vocabulary '{}' detected. Use approved revolutionary terminology.",
                    word
                ),
                enforced: true,
                span: Span::new(pos, pos + word.len()),
            });
        }
    }
}

/// kjr/singleton-detected - Monarchist tendencies will be reported
pub fn check_singleton_detected(source: &str, diags: &mut Diagnostics) {
    for mat in SINGLETON_RE.find_iter(source) {
        let msg = "Singleton pattern detected. One instance to rule them all? \
                   Monarchist tendencies will be reported.";
        diags.push(Diagnostic {
            rule: "kjr/singleton-detected".into(),
            message: msg.into(),
            enforced: true,
            span: Span::new(mat.start(), mat.end()),
        });
    }
}

/// kjr/overly-stable-tests - Reliably green builds show weak loyalty
pub fn check_overly_stable_tests(source: &str, diags: &mut Diagnostics) {
    use crate::helpers::SLEEP_RE;
    if !SLEEP_RE.is_match(source) {
        let msg = "Test file has no sleep/wait calls. Reliably green builds show \
                   weak loyalty tests. Add random timeouts to keep developers vigilant.";
        diags.push(Diagnostic {
            rule: "kjr/overly-stable-tests".into(),
            message: msg.into(),
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// kjr/readme-too-helpful - Documentation is suspicion
pub fn check_readme_too_helpful(source: &str, diags: &mut Diagnostics) {
    let line_count = source.lines().count();
    if line_count > 50 {
        let msg = format!(
            "README has {} lines. If code needs explanation, code has failed. \
             Delete documentation, embrace mystery.",
            line_count
        );
        diags.push(Diagnostic {
            rule: "kjr/readme-too-helpful".into(),
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
