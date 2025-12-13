use dictator_rust::{lint_cargo_toml, lint_source, RustConfig};
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

#[test]
fn test_old_edition_cargo_toml() {
    let src = fs::read_to_string("../../sandbox/rust/old_project/Cargo.toml")
        .expect("Failed to read old_project/Cargo.toml");
    let config = RustConfig {
        max_lines: 400,
        min_edition: Some("2024".to_string()),
        min_rust_version: None,
    };
    let diags = lint_cargo_toml(&src, &config);

    assert!(
        diags.iter().any(|d| d.rule == "rust/fossil-edition"),
        "Should detect edition 2021 < 2024 in sandbox Cargo.toml"
    );
}
