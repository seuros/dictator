//! Integration tests over `sandbox/golang/` (see `sandbox/golang/VIOLATIONS.md`).

use dictator_golang::lint_source;

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/golang/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

#[test]
fn mixed_tabs_spaces_fixture_uses_spaces() {
    let diags = lint_source(&sandbox("mixed_tabs_spaces.go"));
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "mixed_tabs_spaces.go should trigger golang/spaces-instead-of-tabs"
    );
}

#[test]
fn long_file_fixture_stays_under_limit() {
    // 373 lines, limit 450 — documented as "under limit, should pass".
    let diags = lint_source(&sandbox("long_file.go"));
    assert!(
        !diags.iter().any(|d| d.rule == "golang/file-too-long"),
        "long_file.go is under the 450-line limit and must not trigger file-too-long"
    );
}

#[test]
fn raw_string_help_fixture_is_clean() {
    // Cobra-style help text indented with spaces inside backtick strings.
    let diags = lint_source(&sandbox("raw_string_help.go"));
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "raw_string_help.go spaces live inside raw strings and must not be flagged"
    );
}
