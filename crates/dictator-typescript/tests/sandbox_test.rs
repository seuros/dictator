use dictator_typescript::lint_source;
use std::fs;

#[test]
fn test_too_long_file() {
    let src = fs::read_to_string("../../sandbox/typescript/TooLongFile.ts")
        .expect("Failed to read TooLongFile.ts");
    let diags = lint_source(&src);

    assert!(
        diags.iter().any(|d| d.rule == "typescript/file-too-long"),
        "Should detect TooLongFile.ts as too long"
    );
}

#[test]
fn test_wrong_import_order() {
    let src = fs::read_to_string("../../sandbox/typescript/UtilityFunctions.ts")
        .expect("Failed to read UtilityFunctions.ts");
    let diags = lint_source(&src);

    assert!(
        diags.iter().any(|d| d.rule == "typescript/import-order"),
        "Should detect wrong import order in UtilityFunctions.ts"
    );
}

#[test]
fn test_inconsistent_indentation() {
    let src = fs::read_to_string("../../sandbox/typescript/InconsistentIndentation.ts")
        .expect("Failed to read InconsistentIndentation.ts");
    let diags = lint_source(&src);

    assert!(
        diags
            .iter()
            .any(|d| d.rule == "typescript/mixed-indentation"
                || d.rule == "typescript/inconsistent-indentation"),
        "Should detect indentation issues in InconsistentIndentation.ts"
    );
}
