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
mod tests {
    use super::*;
    use dictator_decree_abi::Diagnostics;

    #[test]
    fn detects_unnecessary_mod_rs() {
        use std::fs;
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("dictator_test_mod_rs_solo");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let mod_rs_path = temp_dir.join("mod.rs");
        let mut file = fs::File::create(&mod_rs_path).unwrap();
        writeln!(file, "// Solo mod.rs").unwrap();

        let mut diags = Diagnostics::new();
        check_mod_rs_structure(mod_rs_path.to_str().unwrap(), &mut diags);

        assert!(
            diags.iter().any(|d| d.rule == "rust/unnecessary-mod-rs"),
            "Should detect mod.rs with no siblings"
        );

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn accepts_mod_rs_with_siblings() {
        use std::fs;
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("dictator_test_mod_rs_with_siblings");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let mod_rs_path = temp_dir.join("mod.rs");
        let mut file = fs::File::create(&mod_rs_path).unwrap();
        writeln!(file, "// mod.rs with siblings").unwrap();

        let sibling_path = temp_dir.join("submodule.rs");
        let mut sibling = fs::File::create(&sibling_path).unwrap();
        writeln!(sibling, "// Sibling module").unwrap();

        let mut diags = Diagnostics::new();
        check_mod_rs_structure(mod_rs_path.to_str().unwrap(), &mut diags);

        assert!(
            !diags.iter().any(|d| d.rule == "rust/unnecessary-mod-rs"),
            "Should accept mod.rs with sibling modules"
        );

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn ignores_non_mod_rs_files() {
        use std::fs;
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("dictator_test_regular_rs");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let lib_rs_path = temp_dir.join("lib.rs");
        let mut file = fs::File::create(&lib_rs_path).unwrap();
        writeln!(file, "// Regular file").unwrap();

        let mut diags = Diagnostics::new();
        check_mod_rs_structure(lib_rs_path.to_str().unwrap(), &mut diags);

        assert!(diags.is_empty(), "Should not check non-mod.rs files");

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
