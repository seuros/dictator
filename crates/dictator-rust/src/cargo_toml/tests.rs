use super::*;
use crate::RustConfig;

#[test]
fn detects_edition_too_old() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    let config = RustConfig { min_edition: Some("2024".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        diags.iter().any(|d| d.rule == "rust/fossil-edition"),
        "Should detect edition 2021 < 2024"
    );
}

#[test]
fn accepts_edition_meeting_minimum() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    let config = RustConfig { min_edition: Some("2024".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/fossil-edition"),
        "Should accept edition matching minimum"
    );
}

#[test]
fn accepts_edition_exceeding_minimum() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    let config = RustConfig { min_edition: Some("2021".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/fossil-edition"),
        "Should accept edition exceeding minimum"
    );
}

#[test]
fn detects_missing_edition() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
"#;
    let config = RustConfig { min_edition: Some("2024".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        diags.iter().any(|d| d.rule == "rust/missing-edition"),
        "Should detect missing edition field"
    );
}

#[test]
fn skips_edition_check_when_disabled() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2015"
"#;
    let config = RustConfig { min_edition: None, ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(diags.is_empty(), "Should skip edition check when min_edition is None");
}

#[test]
fn handles_edition_without_spaces() {
    let cargo_toml = r#"[package]
name="test"
edition="2021"
"#;
    let config = RustConfig { min_edition: Some("2024".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        diags.iter().any(|d| d.rule == "rust/fossil-edition"),
        "Should parse edition without spaces around ="
    );
}

#[test]
fn detects_rust_version_too_old() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
rust-version = "1.70"
"#;
    let config = RustConfig { min_rust_version: Some("1.83".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        diags.iter().any(|d| d.rule == "rust/fossil-rust-version"),
        "Should detect rust-version 1.70 < 1.83"
    );
}

#[test]
fn accepts_rust_version_meeting_minimum() {
    let cargo_toml = r#"[package]
name = "test"
rust-version = "1.83"
"#;
    let config = RustConfig { min_rust_version: Some("1.83".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/fossil-rust-version"),
        "Should accept rust-version matching minimum"
    );
}

#[test]
fn accepts_rust_version_exceeding_minimum() {
    let cargo_toml = r#"[package]
name = "test"
rust-version = "1.85"
"#;
    let config = RustConfig { min_rust_version: Some("1.83".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/fossil-rust-version"),
        "Should accept rust-version exceeding minimum"
    );
}

#[test]
fn accepts_rust_version_with_patch() {
    let cargo_toml = r#"[package]
name = "test"
rust-version = "1.83.1"
"#;
    let config = RustConfig { min_rust_version: Some("1.83.0".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/fossil-rust-version"),
        "Should accept 1.83.1 >= 1.83.0"
    );
}

#[test]
fn detects_missing_rust_version() {
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
"#;
    let config = RustConfig { min_rust_version: Some("1.83".to_string()), ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        diags.iter().any(|d| d.rule == "rust/missing-rust-version"),
        "Should detect missing rust-version field"
    );
}

#[test]
fn skips_rust_version_check_when_disabled() {
    let cargo_toml = r#"[package]
name = "test"
rust-version = "1.50"
"#;
    let config = RustConfig { min_rust_version: None, ..Default::default() };
    let diags = lint_cargo_toml(cargo_toml, &config);
    assert!(
        !diags.iter().any(|d| d.rule.contains("rust-version")),
        "Should skip rust-version check when disabled"
    );
}

#[test]
fn version_comparison_works() {
    use std::cmp::Ordering;
    assert_eq!(version_cmp("1.70", "1.83"), Ordering::Less);
    assert_eq!(version_cmp("1.83", "1.83"), Ordering::Equal);
    assert_eq!(version_cmp("1.84", "1.83"), Ordering::Greater);
    assert_eq!(version_cmp("1.83.0", "1.83"), Ordering::Equal);
    assert_eq!(version_cmp("1.83.1", "1.83.0"), Ordering::Greater);
    assert_eq!(version_cmp("2.0", "1.99"), Ordering::Greater);
}
