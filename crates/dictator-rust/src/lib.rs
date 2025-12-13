#![warn(rust_2024_compatibility, clippy::all)]

//! decree.rust - Rust structural rules.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use dictator_supreme::SupremeConfig;
use memchr::memchr_iter;

/// Configuration for rust decree
#[derive(Debug, Clone)]
pub struct RustConfig {
    pub max_lines: usize,
    /// Minimum required Rust edition (e.g., "2024"). None = disabled.
    pub min_edition: Option<String>,
    /// Minimum required rust-version/MSRV (e.g., "1.83"). None = disabled.
    pub min_rust_version: Option<String>,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            max_lines: 400,
            min_edition: None,
            min_rust_version: None,
        }
    }
}

/// Lint Rust source for structural violations.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_config(source, &RustConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &RustConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    check_file_line_count(source, config.max_lines, &mut diags);
    check_visibility_ordering(source, &mut diags);

    diags
}

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
        if trimmed.starts_with("rust-version") && !trimmed.contains(".workspace") {
            if let Some(eq_pos) = trimmed.find('=') {
                let value_part = trimmed[eq_pos + 1..].trim();
                let version = value_part.trim_matches('"').trim_matches('\'').trim();
                let line_start: usize = source.lines().take(line_idx).map(|l| l.len() + 1).sum();
                found_version = Some((version.to_string(), line_start, line_start + line.len()));
                break;
            }
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

/// Rule 1: File line count (ignoring comments and blank lines)
fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let mut code_lines = 0;
    let bytes = source.as_bytes();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        // Count line if it's not blank and not a comment-only line
        if !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            code_lines += 1;
        }

        line_start = nl + 1;
    }

    // Handle last line without newline
    if line_start < bytes.len() {
        let line = &source[line_start..];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            code_lines += 1;
        }
    }

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "rust/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines (max {max_lines}, excl. comments/blanks)"
            ),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// Check if a line is comment-only (// or /* */ style)
fn is_comment_only_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

/// Rule 2: Visibility ordering - pub items should come before private items
fn check_visibility_ordering(source: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut line_start = 0;
    let mut in_struct = false;
    let mut in_impl = false;
    let mut has_private = false;
    let mut in_raw_string = false;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        // Track raw string literals (r" or r#")
        // Simple heuristic: line starting with `let ... = r"` or ending with `";` for multiline
        if !in_raw_string && (trimmed.contains("= r\"") || trimmed.contains("= r#\"")) {
            // Check if the raw string closes on the same line
            let after_open = trimmed.find("= r\"").map_or_else(
                || trimmed.find("= r#\"").map_or("", |pos| &trimmed[pos + 5..]),
                |pos| &trimmed[pos + 4..],
            );
            // If there's no closing quote on this line, we're in a multiline raw string
            if !after_open.contains('"') {
                in_raw_string = true;
            }
        } else if in_raw_string && (trimmed.ends_with("\";") || trimmed == "\";" || trimmed == "\"")
        {
            in_raw_string = false;
            line_start = nl + 1;
            continue;
        }

        // Skip lines inside raw string literals
        if in_raw_string {
            line_start = nl + 1;
            continue;
        }

        // Track struct/impl blocks
        if trimmed.contains("struct ") && trimmed.contains('{') {
            in_struct = true;
            has_private = false;
        } else if trimmed.contains("impl ") && trimmed.contains('{') {
            in_impl = true;
            has_private = false;
        } else if trimmed == "}" || trimmed.starts_with("}\n") {
            in_struct = false;
            in_impl = false;
            has_private = false;
        }

        // Check visibility within struct/impl
        if (in_struct || in_impl) && !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            let is_pub = trimmed.starts_with("pub ");
            let is_field_or_method = is_struct_field_or_impl_item(trimmed);

            if is_field_or_method {
                if !is_pub && !has_private {
                    has_private = true;
                } else if is_pub && has_private {
                    diags.push(Diagnostic {
                        rule: "rust/visibility-order".to_string(),
                        message:
                            "Public item found after private item. Expected all public items first"
                                .to_string(),
                        enforced: false,
                        span: Span::new(line_start, nl),
                    });
                }
            }
        }

        line_start = nl + 1;
    }
}

/// Check if line is a struct field or impl method/associated function
fn is_struct_field_or_impl_item(trimmed: &str) -> bool {
    // Struct fields typically have pattern: [pub] name: Type [,]
    // Impl items typically have pattern: [pub] fn name(...) or [pub] const/type
    // Exclude closing braces, empty lines, attributes, and comments
    if trimmed.is_empty()
        || trimmed == "}"
        || trimmed.starts_with('}')
        || trimmed.starts_with('#')
        || trimmed.starts_with("//")
    {
        return false;
    }

    // Check for impl items (fn, const, type, unsafe, etc.)
    // These are more specific patterns, check them first
    if trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("pub const ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("pub type ")
        || trimmed.starts_with("unsafe fn ")
        || trimmed.starts_with("pub unsafe fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("pub async fn ")
    {
        return true;
    }

    // Check for struct field pattern: identifier followed by colon and type
    // Must start with a valid Rust identifier (letter or underscore, optionally prefixed with pub)
    let field_part = trimmed.strip_prefix("pub ").unwrap_or(trimmed);
    field_part.find(':').is_some_and(|colon_pos| {
        let before_colon = field_part[..colon_pos].trim();
        // Valid field name: starts with letter/underscore, contains only alphanumeric/underscore
        !before_colon.is_empty()
            && before_colon
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && before_colon
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

#[derive(Default)]
pub struct RustDecree {
    config: RustConfig,
    supreme: SupremeConfig,
}

impl RustDecree {
    #[must_use]
    pub const fn new(config: RustConfig, supreme: SupremeConfig) -> Self {
        Self { config, supreme }
    }
}

impl Decree for RustDecree {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn lint(&self, path: &str, source: &str) -> Diagnostics {
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");

        // Cargo.toml gets edition check only (no supreme formatting rules)
        if filename == "Cargo.toml" {
            return lint_cargo_toml(source, &self.config);
        }

        // Regular Rust files get full treatment
        let mut diags = dictator_supreme::lint_source_with_owner(source, &self.supreme, "rust");
        diags.extend(lint_source_with_config(source, &self.config));
        diags
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Rust structural rules".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["rs".to_string()],
            supported_filenames: vec![
                "Cargo.toml".to_string(),
                "build.rs".to_string(),
                "rust-toolchain".to_string(),
                "rust-toolchain.toml".to_string(),
                ".rustfmt.toml".to_string(),
                "rustfmt.toml".to_string(),
                "clippy.toml".to_string(),
                ".clippy.toml".to_string(),
            ],
            skip_filenames: vec!["Cargo.lock".to_string()],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(RustDecree::default())
}

/// Create decree with custom config
#[must_use]
pub fn init_decree_with_config(config: RustConfig) -> BoxDecree {
    Box::new(RustDecree::new(config, SupremeConfig::default()))
}

/// Create decree with custom config + supreme config (merged from decree.supreme + decree.rust)
#[must_use]
pub fn init_decree_with_configs(config: RustConfig, supreme: SupremeConfig) -> BoxDecree {
    Box::new(RustDecree::new(config, supreme))
}

/// Convert `DecreeSettings` to `RustConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> RustConfig {
    RustConfig {
        max_lines: settings.max_lines.unwrap_or(400),
        min_edition: settings.min_edition.clone(),
        min_rust_version: settings.min_rust_version.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_file_too_long() {
        use std::fmt::Write;
        let mut src = String::new();
        for i in 0..410 {
            let _ = writeln!(src, "let x{i} = {i};");
        }
        let diags = lint_source(&src);
        assert!(
            diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "Should detect file with >400 code lines"
        );
    }

    #[test]
    fn ignores_comments_in_line_count() {
        use std::fmt::Write;
        // 390 code lines + 60 comment lines = 450 total, but only 390 counted
        let mut src = String::new();
        for i in 0..390 {
            let _ = writeln!(src, "let x{i} = {i};");
        }
        for i in 0..60 {
            let _ = writeln!(src, "// Comment {i}");
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "Should not count comment-only lines"
        );
    }

    #[test]
    fn ignores_blank_lines_in_count() {
        use std::fmt::Write;
        // 390 code lines + 60 blank lines = 450 total, but only 390 counted
        let mut src = String::new();
        for i in 0..390 {
            let _ = writeln!(src, "let x{i} = {i};");
        }
        for _ in 0..60 {
            src.push('\n');
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "Should not count blank lines"
        );
    }

    #[test]
    fn detects_pub_after_private_in_struct() {
        let src = r"
struct User {
    name: String,
    age: u32,
    pub email: String,
}
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should detect pub field after private fields in struct"
        );
    }

    #[test]
    fn detects_pub_after_private_in_impl() {
        let src = r"
impl User {
    fn private_method(&self) {}
    pub fn public_method(&self) {}
}
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should detect pub method after private method in impl"
        );
    }

    #[test]
    fn accepts_pub_before_private() {
        let src = r"
struct User {
    pub id: u32,
    pub name: String,
    email: String,
}
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should accept public fields before private fields"
        );
    }

    #[test]
    fn accepts_impl_with_correct_order() {
        let src = r"
impl User {
    pub fn new(name: String) -> Self {
        User { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> bool {
        true
    }
}
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should accept impl with public methods before private"
        );
    }

    #[test]
    fn handles_empty_file() {
        let src = "";
        let diags = lint_source(src);
        assert!(diags.is_empty(), "Empty file should have no violations");
    }

    #[test]
    fn handles_file_with_only_comments() {
        let src = "// Comment 1\n// Comment 2\n/* Block comment */\n";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/file-too-long"),
            "File with only comments should not trigger line count"
        );
    }

    // ========== Edition check tests ==========

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

    // ========== Rust-version check tests ==========

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
