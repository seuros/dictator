//! decree.frontmatter - YAML frontmatter structural rules.
//!
//! Applies to files with `---` delimited YAML frontmatter:
//! - Markdown (.md)
//! - MDX (.mdx)
//!
//! Does NOT handle:
//! - Astro (.astro) - uses JS/TS frontmatter, not YAML
//! - Standalone YAML files - use decree.yaml
//! - TOML files - use decree.toml

use std::path::Path;

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use serde::Deserialize;

/// Configuration for the frontmatter decree.
/// Parsed from `.dictate.toml` under `[decree.frontmatter]`.
#[derive(Debug, Clone, Deserialize)]
pub struct FrontmatterConfig {
    /// Expected field order in frontmatter.
    /// Fields not in this list are allowed but not order-checked.
    #[serde(default = "default_order")]
    pub order: Vec<String>,

    /// Required fields that must be present.
    #[serde(default = "default_required")]
    pub required: Vec<String>,
}

fn default_order() -> Vec<String> {
    vec![
        "title".to_string(),
        "description".to_string(),
        "pubDate".to_string(),
    ]
}

fn default_required() -> Vec<String> {
    vec!["title".to_string()]
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        Self {
            order: default_order(),
            required: default_required(),
        }
    }
}

/// Supported YAML frontmatter file extensions.
const FRONTMATTER_EXTENSIONS: &[&str] = &["md", "mdx"];

fn has_frontmatter_extension(file_path: &str) -> bool {
    Path::new(file_path).extension().is_some_and(|ext| {
        let ext_lower = ext.to_ascii_lowercase();
        FRONTMATTER_EXTENSIONS
            .iter()
            .any(|&supported| supported == ext_lower)
    })
}

/// Lint source with default config (for backwards compatibility).
#[must_use]
pub fn lint_source(source: &str, file_path: &str) -> Diagnostics {
    lint_source_with_config(source, file_path, &FrontmatterConfig::default())
}

/// Lint source with custom config.
#[must_use]
pub fn lint_source_with_config(
    source: &str,
    file_path: &str,
    config: &FrontmatterConfig,
) -> Diagnostics {
    let mut diags = Diagnostics::new();

    if has_frontmatter_extension(file_path) {
        check_frontmatter(source, config, &mut diags);
    }

    diags
}

fn check_frontmatter(source: &str, config: &FrontmatterConfig, diags: &mut Diagnostics) {
    // Extract frontmatter between --- markers
    let Some(frontmatter) = extract_frontmatter(source) else {
        return;
    };

    // Parse YAML to get field order
    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&frontmatter.content);
    match parsed {
        Ok(serde_yaml::Value::Mapping(ref mapping)) => {
            check_frontmatter_fields(mapping, frontmatter.start_offset, config, diags);
        }
        Err(e) => {
            diags.push(Diagnostic {
                rule: "decree.frontmatter/invalid-yaml".to_string(),
                message: format!("Invalid YAML frontmatter: {e}"),
                enforced: false,
                span: Span::new(frontmatter.start_offset, frontmatter.end_offset),
            });
        }
        _ => {
            diags.push(Diagnostic {
                rule: "decree.frontmatter/invalid-yaml".to_string(),
                message: "Frontmatter must be a YAML mapping".to_string(),
                enforced: false,
                span: Span::new(frontmatter.start_offset, frontmatter.end_offset),
            });
        }
    }
}

struct ExtractedFrontmatter {
    content: String,
    start_offset: usize,
    end_offset: usize,
}

fn extract_frontmatter(source: &str) -> Option<ExtractedFrontmatter> {
    if !source.starts_with("---") {
        return None;
    }

    let rest = &source[3..];
    let newline_pos = rest.find('\n')?;
    let after_first_marker = &rest[newline_pos + 1..];

    // Find closing marker
    after_first_marker.find("---").map(|closing_pos| {
        let content = after_first_marker[..closing_pos].to_string();
        let start_offset = 3 + newline_pos + 1;
        let end_offset = start_offset + closing_pos;

        ExtractedFrontmatter {
            content,
            start_offset,
            end_offset,
        }
    })
}

fn check_frontmatter_fields(
    mapping: &serde_yaml::Mapping,
    start_offset: usize,
    config: &FrontmatterConfig,
    diags: &mut Diagnostics,
) {
    // Check required fields from config
    for field in &config.required {
        let key = serde_yaml::Value::String(field.clone());
        if !mapping.contains_key(&key) {
            diags.push(Diagnostic {
                rule: "decree.frontmatter/missing-required-field".to_string(),
                message: format!("Missing required field: {field}"),
                enforced: false,
                span: Span::new(start_offset, start_offset),
            });
        }
    }

    // Check field order from config
    if config.order.is_empty() {
        return;
    }

    let mut last_order_index: Option<usize> = None;
    for (key, _value) in mapping {
        if let serde_yaml::Value::String(key_str) = key
            && let Some(order_index) = config.order.iter().position(|f| f == key_str)
        {
            if let Some(last_idx) = last_order_index
                && order_index < last_idx
            {
                diags.push(Diagnostic {
                    rule: "decree.frontmatter/field-order".to_string(),
                    message: format!(
                        "Field '{}' should come before '{}' (expected order: {})",
                        key_str,
                        config.order[last_idx],
                        config.order.join(", ")
                    ),
                    enforced: true,
                    span: Span::new(start_offset, start_offset),
                });
            }
            last_order_index = Some(order_index);
        }
    }
}

/// The frontmatter decree plugin.
#[derive(Default)]
pub struct Frontmatter {
    config: FrontmatterConfig,
}

impl Frontmatter {
    /// Create a new frontmatter plugin with custom config.
    #[must_use]
    pub const fn with_config(config: FrontmatterConfig) -> Self {
        Self { config }
    }
}

impl Decree for Frontmatter {
    fn name(&self) -> &'static str {
        "frontmatter"
    }

    fn lint(&self, path: &str, source: &str) -> Diagnostics {
        lint_source_with_config(source, path, &self.config)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Frontmatter field ordering and validation".to_string(),
            persona: "The Registrar".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["md".to_string(), "mdx".to_string(), "astro".to_string()],
            supported_filenames: vec![],
            skip_filenames: vec![],
            file_scope_rules: vec!["missing-required-field".to_string()],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

/// Create plugin with default config.
#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(Frontmatter::default())
}

/// Create plugin with custom config.
#[must_use]
pub fn init_decree_with_config(config: FrontmatterConfig) -> BoxDecree {
    Box::new(Frontmatter::with_config(config))
}

/// Convert `DecreeSettings` from .dictate.toml to `FrontmatterConfig`.
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> FrontmatterConfig {
    FrontmatterConfig {
        order: settings.order.clone().unwrap_or_else(default_order),
        required: settings.required.clone().unwrap_or_else(default_required),
    }
}

#[cfg(test)]
mod tests;
