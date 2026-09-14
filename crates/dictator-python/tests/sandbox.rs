//! Integration tests over `sandbox/python/` (see `sandbox/EXPECTED_VIOLATIONS.md`).

use dictator_python::lint_source;

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/python/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

#[test]
fn wrong_import_order_fixture() {
    let diags = lint_source(&sandbox("wrong_import_order.py"));
    assert!(
        diags.iter().any(|d| d.rule == "python/import-order"),
        "wrong_import_order.py should trigger python/import-order"
    );
}

#[test]
fn mixed_indentation_fixture() {
    let diags = lint_source(&sandbox("mixed_indentation.py"));
    assert!(
        diags.iter().any(|d| d.rule.starts_with("python/")),
        "mixed_indentation.py should trigger a python indentation rule"
    );
}

#[test]
fn long_file_fixture_counts_code_lines_only() {
    // 450 physical lines, but only 337 code lines (blanks/comments excluded),
    // which is under the 380 limit. EXPECTED_VIOLATIONS.md's "450 > 380" claim
    // predates code-line counting; this file's real job is mixed line endings.
    let diags = lint_source(&sandbox("long_file_mixed_endings.py"));
    assert!(
        !diags.iter().any(|d| d.rule == "python/file-too-long"),
        "long_file_mixed_endings.py has 337 code lines and must not trigger file-too-long"
    );
}
