//! Integration tests over `sandbox/typescript/` (see `sandbox/typescript/VIOLATIONS.md`).

use dictator_typescript::lint_source;

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/typescript/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

#[test]
fn too_long_file_fixture() {
    let diags = lint_source(&sandbox("TooLongFile.ts"));
    assert!(
        diags.iter().any(|d| d.rule == "typescript/file-too-long"),
        "TooLongFile.ts should trigger typescript/file-too-long"
    );
}

#[test]
fn wrong_import_order_fixture() {
    let diags = lint_source(&sandbox("UtilityFunctions.ts"));
    assert!(
        diags.iter().any(|d| d.rule == "typescript/import-order"),
        "UtilityFunctions.ts should trigger typescript/import-order"
    );
}

#[test]
fn inconsistent_indentation_fixture() {
    let diags = lint_source(&sandbox("InconsistentIndentation.ts"));
    assert!(
        diags.iter().any(|d| d.rule == "typescript/inconsistent-indentation"),
        "InconsistentIndentation.ts should trigger typescript/inconsistent-indentation"
    );
}
