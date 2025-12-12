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
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["md".to_string(), "mdx".to_string(), "astro".to_string()],
            supported_filenames: vec![],
            skip_filenames: vec![],
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
mod tests {
    use super::*;

    #[test]
    fn valid_frontmatter_order() {
        // Default order is: title, description, pubDate
        let src =
            "---\ntitle: Test\ndescription: A description\npubDate: 2024-01-01\n---\n# Content\n";
        let diags = lint_source(src, "test.md");
        assert!(
            diags.is_empty(),
            "Expected no diagnostics for valid frontmatter"
        );
    }

    #[test]
    fn detects_wrong_field_order() {
        // Default order is: title, description, pubDate
        // This has pubDate before title - wrong order
        let src = "---\npubDate: 2024-01-01\ndescription: Test desc\ntitle: Test\n---\n# Content\n";
        let diags = lint_source(src, "test.md");
        assert!(
            !diags.is_empty(),
            "Expected diagnostics for wrong field order"
        );
        assert_eq!(diags[0].rule, "decree.frontmatter/field-order");
    }

    #[test]
    fn detects_missing_required_fields() {
        // Use custom config that requires both title and slug
        let config = FrontmatterConfig {
            order: vec!["title".to_string(), "slug".to_string()],
            required: vec!["title".to_string(), "slug".to_string()],
        };
        let src = "---\ntitle: Test\n---\n# Content\n";
        let diags = lint_source_with_config(src, "test.md", &config);
        assert!(
            !diags.is_empty(),
            "Expected diagnostics for missing required field"
        );
        let has_missing_slug = diags.iter().any(|d| {
            d.rule == "decree.frontmatter/missing-required-field" && d.message.contains("slug")
        });
        assert!(has_missing_slug);
    }

    #[test]
    fn respects_custom_config() {
        // Custom config with different field order
        let config = FrontmatterConfig {
            order: vec![
                "title".to_string(),
                "description".to_string(),
                "pubDate".to_string(),
                "author".to_string(),
            ],
            required: vec!["title".to_string(), "description".to_string()],
        };

        // Valid order per custom config
        let src = "---\ntitle: Test\ndescription: A test\npubDate: 2024-01-01\n---\n# Content\n";
        let diags = lint_source_with_config(src, "test.md", &config);
        assert!(
            diags.is_empty(),
            "Expected no errors for valid custom order"
        );

        // Wrong order per custom config
        let src_wrong = "---\npubDate: 2024-01-01\ntitle: Test\n---\n# Content\n";
        let diags_wrong = lint_source_with_config(src_wrong, "test.md", &config);
        assert!(
            diags_wrong
                .iter()
                .any(|d| d.rule == "decree.frontmatter/field-order"),
            "Expected field order violation"
        );

        // Missing required field
        let src_missing = "---\ntitle: Test\n---\n# Content\n";
        let diags_missing = lint_source_with_config(src_missing, "test.md", &config);
        assert!(
            diags_missing
                .iter()
                .any(|d| d.rule == "decree.frontmatter/missing-required-field"
                    && d.message.contains("description")),
            "Expected missing description error"
        );
    }

    #[test]
    fn ignores_non_markdown_files() {
        let src = "title: Test\nslug: test\n";
        let diags = lint_source(src, "test.txt");
        assert!(diags.is_empty());
    }

    #[test]
    fn supports_mdx_files() {
        let src = "---\ntitle: Test\nslug: test-slug\npubDate: 2024-01-01\n---\n\nimport Component from './Component';\n\n# Content\n";
        let diags = lint_source(src, "test.mdx");
        assert!(
            diags.is_empty(),
            "Expected no diagnostics for valid MDX frontmatter"
        );
    }

    #[test]
    fn ignores_yaml_files() {
        // YAML files are NOT frontmatter - they're standalone config files
        let src = "---\ntitle: Test\nslug: test\n---\n";
        let diags = lint_source(src, "config.yml");
        assert!(
            diags.is_empty(),
            "decree.frontmatter should not lint .yml files"
        );
    }

    #[test]
    fn ignores_toml_files() {
        // TOML files are NOT frontmatter
        let src = "[package]\nname = \"test\"\n";
        let diags = lint_source(src, "Cargo.toml");
        assert!(
            diags.is_empty(),
            "decree.frontmatter should not lint .toml files"
        );
    }

    #[test]
    fn ignores_astro_files() {
        // Astro files have JS/TS frontmatter, not YAML
        let src = "---\nconst title = 'Test';\n---\n<html>{title}</html>\n";
        let diags = lint_source(src, "page.astro");
        assert!(
            diags.is_empty(),
            "decree.frontmatter should not lint .astro files"
        );
    }

    #[test]
    fn handles_markdown_without_frontmatter() {
        let src = "# Content\nNo frontmatter here\n";
        let diags = lint_source(src, "test.md");
        assert!(diags.is_empty());
    }

    #[test]
    fn detects_invalid_yaml() {
        let src = "---\ntitle: [broken yaml\n---\n# Content\n";
        let diags = lint_source(src, "test.md");
        assert!(!diags.is_empty());
        assert_eq!(diags[0].rule, "decree.frontmatter/invalid-yaml");
    }

    #[test]
    fn sandbox_blog_wrong_order() {
        // Test the actual sandbox file case: pubDate comes before title
        // Default order: title, description, pubDate
        // This frontmatter has: pubDate, description, title (wrong!)
        let src = "---\npubDate: 2024-12-01\ndescription: This blog post has wrong frontmatter ordering\ntitle: Blog Post With Wrong Frontmatter Order\nauthor: John Doe\n---\n\n# Blog Post Content\n";
        let diags = lint_source(src, "blog-wrong-frontmatter-order.md");
        assert!(
            !diags.is_empty(),
            "Expected to detect field order violation"
        );

        assert!(
            diags
                .iter()
                .any(|d| d.rule == "decree.frontmatter/field-order"),
            "Expected field order violation diagnostic"
        );
    }
}
