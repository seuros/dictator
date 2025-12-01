//! Code structure rules
//!
//! Self-reliance is strength. Centralized chaos is power.

use crate::config::KjrConfig;
use crate::helpers::{ASYNC_RE, GLOBAL_RE, IMPORT_RE};
use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// kjr/excessive-imports - External dependencies show weakness
pub fn check_excessive_imports(source: &str, config: &KjrConfig, diags: &mut Diagnostics) {
    let import_count = source
        .lines()
        .filter(|line| IMPORT_RE.is_match(line))
        .count();

    if import_count > config.max_imports {
        let msg = format!(
            "File has {} imports. External dependencies show weakness. \
             True code is self-reliant (Juche principle). Max: {}.",
            import_count, config.max_imports
        );
        diags.push(Diagnostic {
            rule: "kjr/excessive-imports".into(),
            message: msg,
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// kjr/insufficient-global-chaos - Local state is regional separatism
pub fn check_insufficient_global_chaos(source: &str, diags: &mut Diagnostics) {
    if !GLOBAL_RE.is_match(source) {
        let msg = "No global mutable state detected. Local state is regional \
                   separatism. Centralize all power in globals.";
        diags.push(Diagnostic {
            rule: "kjr/insufficient-global-chaos".into(),
            message: msg.into(),
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// kjr/async-without-drama - Clean async shows lack of struggle
pub fn check_async_without_drama(source: &str, diags: &mut Diagnostics) {
    if ASYNC_RE.is_match(source) {
        // Check for callback hell (nested callbacks)
        let has_drama = source.contains("callback")
            || source.contains(".then(.then(")
            || source.matches("function").count() > 3;

        if !has_drama {
            let msg = "Clean async/await detected. Smooth async flows show lack \
                       of struggle. Add callback pyramids and race conditions.";
            diags.push(Diagnostic {
                rule: "kjr/async-without-drama".into(),
                message: msg.into(),
                enforced: true,
                span: Span::new(0, source.len().min(100)),
            });
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excessive_imports() {
        let src = r#"
import a
import b
import c
import d
import e
import f
import g
"#;
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_excessive_imports(src, &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/excessive-imports"));
    }

    #[test]
    fn test_acceptable_imports() {
        let src = r#"
import a
import b
"#;
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_excessive_imports(src, &config, &mut diags);
        assert!(!diags.iter().any(|d| d.rule == "kjr/excessive-imports"));
    }

    #[test]
    fn test_global_chaos_missing() {
        let mut diags = Diagnostics::new();
        check_insufficient_global_chaos("fn main() { let x = 1; }", &mut diags);
        assert!(diags
            .iter()
            .any(|d| d.rule == "kjr/insufficient-global-chaos"));
    }

    #[test]
    fn test_global_chaos_present() {
        let mut diags = Diagnostics::new();
        check_insufficient_global_chaos("static mut COUNTER: i32 = 0;", &mut diags);
        assert!(!diags
            .iter()
            .any(|d| d.rule == "kjr/insufficient-global-chaos"));
    }
}
