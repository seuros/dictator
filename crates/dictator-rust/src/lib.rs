#![warn(rust_2024_compatibility, clippy::all)]

//! decree.rust - Rust structural rules.

mod cargo_toml;
mod counting;
mod structure;
mod visibility;

use dictator_decree_abi::{BoxDecree, Decree, Diagnostics};
use dictator_supreme::SupremeConfig;

pub use cargo_toml::lint_cargo_toml;

/// Configuration for rust decree
#[derive(Debug, Clone)]
pub struct RustConfig {
    pub max_lines: usize,
    /// Minimum required Rust edition (e.g., "2024"). None = disabled.
    pub min_edition: Option<String>,
    /// Minimum required rust-version/MSRV (e.g., "1.83"). None = disabled.
    pub min_rust_version: Option<String>,
    /// When true, `line-too-long` violations on comment lines are suppressed.
    pub ignore_comments: bool,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            max_lines: 400,
            min_edition: None,
            min_rust_version: None,
            ignore_comments: false,
        }
    }
}

/// Lint Rust source for structural violations.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_configs(source, &RustConfig::default(), &SupremeConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &RustConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    counting::check_file_line_count(source, config.max_lines, &mut diags);
    visibility::check_visibility_ordering(source, &mut diags);

    diags
}

/// Lint with custom config + supreme config (merged from decree.supreme + decree.rust)
#[must_use]
pub fn lint_source_with_configs(
    source: &str,
    rust_config: &RustConfig,
    supreme_config: &SupremeConfig,
) -> Diagnostics {
    let mut diags = Diagnostics::new();

    let supreme_diags = dictator_supreme::lint_source_with_owner(source, supreme_config, "rust");

    if rust_config.ignore_comments {
        // Filter out line-too-long violations on comment lines
        let lines: Vec<&str> = source.lines().collect();
        diags.extend(supreme_diags.into_iter().filter(|d| {
            if d.rule == "rust/line-too-long" {
                let line_idx = source[..d.span.start].matches('\n').count();
                !lines
                    .get(line_idx)
                    .is_some_and(|line| line.trim_start().starts_with("//"))
            } else {
                true
            }
        }));
    } else {
        diags.extend(supreme_diags);
    }

    // Rust-specific rules
    diags.extend(lint_source_with_config(source, rust_config));

    diags
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
            return cargo_toml::lint_cargo_toml(source, &self.config);
        }

        // Regular Rust files get full treatment
        let mut diags = lint_source_with_configs(source, &self.config, &self.supreme);

        // Check mod.rs structure (needs filesystem access)
        structure::check_mod_rs_structure(path, &mut diags);

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
        ignore_comments: settings.ignore_comments.unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_long_comment_lines_when_configured() {
        let long_comment = format!("// {}\n", "x".repeat(150));
        let src = format!("fn main() {{\n{long_comment}}}\n");
        let config = RustConfig {
            ignore_comments: true,
            ..Default::default()
        };
        let supreme = SupremeConfig {
            max_line_length: Some(120),
            ..Default::default()
        };
        let diags = lint_source_with_configs(&src, &config, &supreme);
        assert!(!diags.iter().any(|d| d.rule == "rust/line-too-long"));
    }

    #[test]
    fn detects_long_comment_lines_when_not_configured() {
        let long_comment = format!("// {}\n", "x".repeat(150));
        let src = format!("fn main() {{\n{long_comment}}}\n");
        let config = RustConfig::default(); // ignore_comments = false
        let supreme = SupremeConfig {
            max_line_length: Some(120),
            ..Default::default()
        };
        let diags = lint_source_with_configs(&src, &config, &supreme);
        assert!(diags.iter().any(|d| d.rule == "rust/line-too-long"));
    }

    #[test]
    fn still_detects_long_code_lines_with_ignore_comments() {
        let long_code = format!("    let x = \"{}\";\n", "a".repeat(150));
        let src = format!("fn main() {{\n{long_code}}}\n");
        let config = RustConfig {
            ignore_comments: true,
            ..Default::default()
        };
        let supreme = SupremeConfig {
            max_line_length: Some(120),
            ..Default::default()
        };
        let diags = lint_source_with_configs(&src, &config, &supreme);
        assert!(diags.iter().any(|d| d.rule == "rust/line-too-long"));
    }
}
