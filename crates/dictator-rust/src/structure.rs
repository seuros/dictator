//! Filesystem-based structural checks.

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// Check if mod.rs should be refactored to file.rs
pub fn check_mod_rs_structure(path: &str, diags: &mut Diagnostics) {
    let path = std::path::Path::new(path);

    // Only check files named mod.rs
    if path.file_name().and_then(|n| n.to_str()) != Some("mod.rs") {
        return;
    }

    // Get parent directory
    let Some(parent) = path.parent() else {
        return;
    };

    // Count .rs files in the same directory (excluding mod.rs itself)
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };

    let sibling_count = entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().extension().and_then(|ext| ext.to_str()) == Some("rs")
                && e.file_name() != "mod.rs"
        })
        .count();

    // Violation: mod.rs with no siblings should be file.rs
    if sibling_count == 0 {
        let module_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("module");

        diags.push(Diagnostic {
            rule: "rust/unnecessary-mod-rs".to_string(),
            message: format!(
                "mod.rs with no submodules should be {module_name}.rs - \
                 refactor when you need it, inshallah"
            ),
            enforced: false,
            span: Span::new(0, 100),
        });
    }
}

#[cfg(test)]
mod tests;
