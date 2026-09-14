//! Integration tests over `sandbox/rust/` (see `sandbox/rust/INDEX.md`).

use dictator_rust::{RustConfig, lint_cargo_toml, lint_source};

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/rust/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

#[test]
fn visibility_order_fixture() {
    let diags = lint_source(&sandbox("04_visibility_order.rs"));
    assert!(
        diags.iter().any(|d| d.rule == "rust/visibility-order"),
        "04_visibility_order.rs should trigger rust/visibility-order"
    );
}

#[test]
fn long_file_fixture_exceeds_limit() {
    // 534 lines, limit 400.
    let diags = lint_source(&sandbox("05_long_file.rs"));
    assert!(
        diags.iter().any(|d| d.rule == "rust/file-too-long"),
        "05_long_file.rs should trigger rust/file-too-long"
    );
}

#[test]
fn old_edition_cargo_toml_fixture() {
    // Edition checks are opt-in; the fixture declares edition 2021.
    let config = RustConfig { min_edition: Some("2024".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(&sandbox("old_edition_cargo.toml"), &config);
    assert!(
        diags.iter().any(|d| d.rule == "rust/fossil-edition"),
        "old_edition_cargo.toml declares edition 2021 and should trigger rust/fossil-edition, got: {:?}",
        diags.iter().map(|d| &d.rule).collect::<Vec<_>>()
    );
}
