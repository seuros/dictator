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

    counting::check_file_line_count(source, config.max_lines, &mut diags);
    visibility::check_visibility_ordering(source, &mut diags);

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
        let mut diags = dictator_supreme::lint_source_with_owner(source, &self.supreme, "rust");
        diags.extend(lint_source_with_config(source, &self.config));

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
    }
}
