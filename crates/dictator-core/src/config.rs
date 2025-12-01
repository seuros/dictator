//! Configuration loading for .dictate.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration from .dictate.toml
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DictateConfig {
    #[serde(default)]
    pub decree: HashMap<String, DecreeSettings>,
}

/// Settings for a specific decree (language)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DecreeSettings {
    // Custom decree loading (for WASM/native plugins)
    pub enabled: Option<bool>,
    pub path: Option<String>,

    // Supreme decree settings
    pub trailing_whitespace: Option<String>,
    pub tabs_vs_spaces: Option<String>,
    pub tab_width: Option<usize>,
    pub final_newline: Option<String>,
    pub line_endings: Option<String>,
    pub max_line_length: Option<usize>,
    pub blank_line_whitespace: Option<String>,

    // Language-specific settings
    pub max_lines: Option<usize>,
    pub ignore_comments: Option<bool>,
    pub ignore_blank_lines: Option<bool>,
    pub method_visibility_order: Option<Vec<String>>,
    pub comment_spacing: Option<bool>,
    pub import_order: Option<Vec<String>>,
    pub visibility_order: Option<Vec<String>>,

    // Frontmatter decree settings
    pub order: Option<Vec<String>>,
    pub required: Option<Vec<String>>,

    // Linter integration (for supremecourt mode)
    pub linter: Option<LinterConfig>,
}

/// External linter configuration
///
/// Only specify the command - Dictator controls the args for:
/// - Autofix mode (-A, --fix, etc.)
/// - Parseable output format (--format json, etc.)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinterConfig {
    pub command: String,
}

impl DictateConfig {
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the TOML content is invalid.
    pub fn from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse TOML: {e}"),
            )
        })
    }

    /// Load from default location (.dictate.toml in current directory)
    #[must_use]
    pub fn load_default() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        let config_path = cwd.join(".dictate.toml");

        if !config_path.exists() {
            return None;
        }

        Self::from_file(&config_path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_config() {
        let toml = r#"
[decree.supreme]
trailing_whitespace = "deny"
tabs_vs_spaces = "spaces"
tab_width = 2
final_newline = "require"
line_endings = "lf"
max_line_length = 120
blank_line_whitespace = "deny"

[decree.ruby]
max_lines = 300
ignore_comments = true
ignore_blank_lines = true
method_visibility_order = ["public", "protected", "private"]
comment_spacing = true

[decree.typescript]
max_lines = 350
ignore_comments = true
ignore_blank_lines = true
import_order = ["system", "external", "internal"]
"#;

        let config: DictateConfig = toml::from_str(toml).unwrap();
        assert!(config.decree.contains_key("supreme"));
        assert!(config.decree.contains_key("ruby"));
        assert!(config.decree.contains_key("typescript"));

        let supreme = config.decree.get("supreme").unwrap();
        assert_eq!(supreme.max_line_length, Some(120));
        assert_eq!(supreme.tabs_vs_spaces, Some("spaces".to_string()));

        let ruby = config.decree.get("ruby").unwrap();
        assert_eq!(ruby.max_lines, Some(300));
        assert_eq!(ruby.ignore_comments, Some(true));

        let ts = config.decree.get("typescript").unwrap();
        assert_eq!(ts.max_lines, Some(350));
    }
}
