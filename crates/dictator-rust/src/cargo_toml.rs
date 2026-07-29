//! Cargo.toml manifest linting.

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

use crate::RustConfig;

/// Lint Cargo.toml for edition and rust-version compliance
#[must_use]
pub fn lint_cargo_toml(source: &str, config: &RustConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    if let Some(ref min_edition) = config.min_edition {
        check_cargo_edition(source, min_edition, &mut diags);
    }

    if let Some(ref min_rust_version) = config.min_rust_version {
        check_rust_version(source, min_rust_version, &mut diags);
    }

    diags
}

/// Check Cargo.toml edition against minimum required
fn check_cargo_edition(source: &str, min_edition: &str, diags: &mut Diagnostics) {
    // Simple line-based parsing to find edition
    let mut found_edition: Option<(String, usize, usize)> = None;

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Skip workspace inheritance: edition.workspace = true
        if trimmed.starts_with("edition.workspace") {
            return; // Can't validate without parsing workspace Cargo.toml
        }
        if trimmed.starts_with("edition") && !trimmed.contains(".workspace") {
            // Parse: edition = "2021" or edition="2021"
            if let Some(eq_pos) = trimmed.find('=') {
                let value_part = trimmed[eq_pos + 1..].trim();
                let edition = value_part.trim_matches('"').trim_matches('\'').trim();
                let line_start: usize = source.lines().take(line_idx).map(|l| l.len() + 1).sum();
                found_edition = Some((edition.to_string(), line_start, line_start + line.len()));
                break;
            }
        }
    }

    match found_edition {
        Some((edition, start, end)) => {
            if edition_ord(&edition) < edition_ord(min_edition) {
                diags.push(Diagnostic {
                    rule: "rust/fossil-edition".to_string(),
                    message: format!(
                        "edition {edition} is fossilized, the Dictator demands {min_edition}"
                    ),
                    enforced: true,
                    span: Span::new(start, end),
                });
            }
        }
        None => {
            diags.push(Diagnostic {
                rule: "rust/missing-edition".to_string(),
                message: format!("no edition declared, the Dictator demands {min_edition}"),
                enforced: false,
                span: Span::new(0, source.len().min(50)),
            });
        }
    }
}

/// Convert edition string to comparable ordinal
fn edition_ord(edition: &str) -> u32 {
    match edition {
        "2015" => 1,
        "2018" => 2,
        "2021" => 3,
        "2024" => 4,
        _ => 0,
    }
}

/// Check Cargo.toml rust-version against minimum required
fn check_rust_version(source: &str, min_version: &str, diags: &mut Diagnostics) {
    let mut found_version: Option<(String, usize, usize)> = None;

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Skip workspace inheritance: rust-version.workspace = true
        if trimmed.starts_with("rust-version.workspace") {
            return; // Can't validate without parsing workspace Cargo.toml
        }
        if trimmed.starts_with("rust-version")
            && !trimmed.contains(".workspace")
            && let Some(eq_pos) = trimmed.find('=')
        {
            let value_part = trimmed[eq_pos + 1..].trim();
            let version = value_part.trim_matches('"').trim_matches('\'').trim();
            let line_start: usize = source.lines().take(line_idx).map(|l| l.len() + 1).sum();
            found_version = Some((version.to_string(), line_start, line_start + line.len()));
            break;
        }
    }

    match found_version {
        Some((version, start, end)) => {
            if version_cmp(&version, min_version) == std::cmp::Ordering::Less {
                diags.push(Diagnostic {
                    rule: "rust/fossil-rust-version".to_string(),
                    message: format!(
                        "rust-version {version} is prehistoric, the Dictator demands {min_version}+"
                    ),
                    enforced: true,
                    span: Span::new(start, end),
                });
            }
        }
        None => {
            diags.push(Diagnostic {
                rule: "rust/missing-rust-version".to_string(),
                message: format!("no rust-version declared, the Dictator demands {min_version}+"),
                enforced: false,
                span: Span::new(0, source.len().min(50)),
            });
        }
    }
}

/// Compare semver-like versions (1.70 vs 1.83, 1.70.0 vs 1.70.1)
fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|p| p.parse().ok()).collect() };
    let a_parts = parse(a);
    let b_parts = parse(b);

    for i in 0..3 {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => {}
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RustConfig;

    #[test]
    fn detects_edition_too_old() {
        let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
        let config = RustConfig {
            min_edition: Some("2024".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_edition: Some("2024".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_edition: Some("2021".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_edition: Some("2024".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_edition: None,
            ..Default::default()
        };
        let diags = lint_cargo_toml(cargo_toml, &config);
        assert!(
            diags.is_empty(),
            "Should skip edition check when min_edition is None"
        );
    }

    #[test]
    fn handles_edition_without_spaces() {
        let cargo_toml = r#"[package]
name="test"
edition="2021"
"#;
        let config = RustConfig {
            min_edition: Some("2024".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_rust_version: Some("1.83".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_rust_version: Some("1.83".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_rust_version: Some("1.83".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_rust_version: Some("1.83.0".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_rust_version: Some("1.83".to_string()),
            ..Default::default()
        };
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
        let config = RustConfig {
            min_rust_version: None,
            ..Default::default()
        };
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
}
