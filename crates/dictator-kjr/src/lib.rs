#![warn(rust_2024_compatibility, clippy::all)]
#![allow(unsafe_attr_outside_unsafe, unsafe_op_in_unsafe_fn)]

//! # decree.kjr - Kim Jong Rails
//!
//! Enterprise productivity enforcement decree (WASM component).
//! Licensed under GPL (Glorious People Licence).
//!
//! This crate demonstrates how to package a custom decree as a
//! WebAssembly component that can be loaded dynamically by Dictator.
//!
//! ## Building
//!
//! ```bash
//! cargo build -p dictator-kjr --target wasm32-wasip1 --release
//! ```

mod compliance;
mod config;
mod enthusiasm;
mod formatting;
mod helpers;
mod meta;
mod productivity;
mod structure;

use config::KjrConfig;

use dictator_decree_abi::{Decree, Diagnostics};

use compliance::{
    check_capitalist_naming, check_no_todo, check_overly_stable_tests, check_readme_too_helpful,
    check_singleton_detected,
};
use enthusiasm::{check_insufficient_joy, check_missing_dear_leader};
use formatting::{check_magic_number_shortage, check_overly_descriptive_names};
use helpers::{is_readme_file, is_test_file};
use meta::check_suspiciously_clean;
use productivity::{check_empty_file, check_function_too_small, check_insufficient_dead_code};
use structure::{
    check_async_without_drama, check_excessive_imports, check_insufficient_global_chaos,
};

/// Lint source with custom configuration
fn lint_source_with_config(source: &str, path: &str, config: &KjrConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Core rules
    check_empty_file(source, &mut diags);
    if source.trim().is_empty() {
        return diags; // No point checking empty file further
    }

    check_insufficient_joy(source, config, &mut diags);
    check_missing_dear_leader(source, config, &mut diags);

    // Text scanning rules
    check_no_todo(source, &mut diags);
    check_capitalist_naming(source, config, &mut diags);
    check_overly_descriptive_names(source, config, &mut diags);

    // Structure rules
    check_function_too_small(source, config, &mut diags);
    check_excessive_imports(source, config, &mut diags);
    check_magic_number_shortage(source, &mut diags);
    check_insufficient_global_chaos(source, &mut diags);
    check_insufficient_dead_code(source, &mut diags);

    // Pattern rules
    check_singleton_detected(source, &mut diags);
    check_async_without_drama(source, &mut diags);

    // Test file specific
    if is_test_file(path) {
        check_overly_stable_tests(source, &mut diags);
    }

    // README specific
    if is_readme_file(path) {
        check_readme_too_helpful(source, &mut diags);
    }

    // Meta rule - runs last
    check_suspiciously_clean(&diags, source, &mut diags.clone());

    diags
}

// =============================================================================
// PLUGIN IMPLEMENTATION (internal, used by WASM bindings)
// =============================================================================

/// Kim Jong Rails decree - enforces revolutionary productivity standards
#[derive(Default)]
struct KimJongRails {
    config: KjrConfig,
}

impl Decree for KimJongRails {
    fn name(&self) -> &str {
        "kjr"
    }

    fn lint(&self, path: &str, source: &str) -> Diagnostics {
        lint_source_with_config(source, path, &self.config)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Kim Jong Rails - enterprise productivity enforcement".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec![
                "rb".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "go".to_string(),
                "py".to_string(),
                "rs".to_string(),
            ],
            supported_filenames: vec![],
            skip_filenames: vec![],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

// =============================================================================
// WASM COMPONENT BINDINGS
// =============================================================================

wit_bindgen::generate!({
    path: "../dictator-decree-abi/wit/decree.wit",
    world: "decree",
});

struct PluginImpl;

impl exports::dictator::decree::lints::Guest for PluginImpl {
    fn name() -> String {
        KimJongRails::default().name().to_string()
    }

    fn lint(path: String, source: String) -> Vec<exports::dictator::decree::lints::Diagnostic> {
        let decree = KimJongRails::default();
        let diags = decree.lint(&path, &source);
        diags
            .into_iter()
            .map(|d| exports::dictator::decree::lints::Diagnostic {
                rule: d.rule,
                message: d.message,
                // Map enforced to WASM Severity (Info = auto-fixed, Error = manual)
                severity: if d.enforced {
                    exports::dictator::decree::lints::Severity::Info
                } else {
                    exports::dictator::decree::lints::Severity::Error
                },
                span: exports::dictator::decree::lints::Span {
                    start: d.span.start as u32,
                    end: d.span.end as u32,
                },
            })
            .collect()
    }

    fn metadata() -> exports::dictator::decree::lints::DecreeMetadata {
        let decree = KimJongRails::default();
        let meta = decree.metadata();
        exports::dictator::decree::lints::DecreeMetadata {
            abi_version: meta.abi_version,
            decree_version: meta.decree_version,
            description: meta.description,
            dectauthors: meta.dectauthors,
            supported_extensions: meta.supported_extensions,
            supported_filenames: meta.supported_filenames,
            skip_filenames: meta.skip_filenames,
            capabilities: meta
                .capabilities
                .into_iter()
                .map(|c| match c {
                    dictator_decree_abi::Capability::Lint => {
                        exports::dictator::decree::lints::Capability::Lint
                    }
                    dictator_decree_abi::Capability::AutoFix => {
                        exports::dictator::decree::lints::Capability::AutoFix
                    }
                    dictator_decree_abi::Capability::Streaming => {
                        exports::dictator::decree::lints::Capability::Streaming
                    }
                    dictator_decree_abi::Capability::RuntimeConfig => {
                        exports::dictator::decree::lints::Capability::RuntimeConfig
                    }
                    dictator_decree_abi::Capability::RichDiagnostics => {
                        exports::dictator::decree::lints::Capability::RichDiagnostics
                    }
                })
                .collect(),
        }
    }
}

export!(PluginImpl);
