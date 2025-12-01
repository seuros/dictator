use dictator_rust::lint_source;
use std::fs;

#[test]
fn test_too_long_file() {
    let src = fs::read_to_string("../../sandbox/rust/05_long_file.rs")
        .expect("Failed to read 05_long_file.rs");
    let diags = lint_source(&src);

    assert!(
        diags.iter().any(|d| d.rule == "rust/file-too-long"),
        "Should detect 05_long_file.rs as too long"
    );
}
