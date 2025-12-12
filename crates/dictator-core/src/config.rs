//! Configuration loading for .dictate.toml

use garde::Validate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration from .dictate.toml
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DictateConfig {
    #[serde(default)]
    pub decree: HashMap<String, DecreeSettings>,
}

/// Settings for a specific decree (language)
#[derive(Debug, Clone, Default, Deserialize, Serialize, Validate)]
#[garde(context(()))]
pub struct DecreeSettings {
    // Custom decree loading (for WASM/native plugins)
    #[garde(skip)]
    pub enabled: Option<bool>,
    #[garde(skip)]
    pub path: Option<String>,

    // Supreme decree settings
    #[garde(custom(validate_whitespace_policy))]
    pub trailing_whitespace: Option<String>,
    #[garde(custom(validate_tabs_vs_spaces))]
    pub tabs_vs_spaces: Option<String>,
    #[garde(custom(validate_tab_width))]
    pub tab_width: Option<usize>,
    #[garde(custom(validate_newline_policy))]
    pub final_newline: Option<String>,
    #[garde(custom(validate_line_endings))]
    pub line_endings: Option<String>,
    #[garde(custom(validate_max_line_length))]
    pub max_line_length: Option<usize>,
    #[garde(custom(validate_whitespace_policy))]
    pub blank_line_whitespace: Option<String>,

    // Language-specific settings
    #[garde(custom(validate_max_lines))]
    pub max_lines: Option<usize>,
    #[garde(skip)]
    pub ignore_comments: Option<bool>,
    #[garde(skip)]
    pub ignore_blank_lines: Option<bool>,
    #[garde(skip)]
    pub method_visibility_order: Option<Vec<String>>,
    #[garde(skip)]
    pub comment_spacing: Option<bool>,
    #[garde(skip)]
    pub import_order: Option<Vec<String>>,
    #[garde(skip)]
    pub visibility_order: Option<Vec<String>>,

    // Frontmatter decree settings
    #[garde(skip)]
    pub order: Option<Vec<String>>,
    #[garde(skip)]
    pub required: Option<Vec<String>>,

    // Linter integration (for supremecourt mode)
    #[garde(skip)]
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

// ============================================================================
// Custom Validators
// Note: garde requires `&Option<T>` and `&()` signatures - clippy lints suppressed
// ============================================================================

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_whitespace_policy(value: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        match v.as_str() {
            "deny" | "allow" => Ok(()),
            _ => Err(garde::Error::new(format!(
                "'{v}' is not a valid policy - try 'deny' or 'allow'"
            ))),
        }
    } else {
        Ok(())
    }
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_tabs_vs_spaces(value: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        match v.as_str() {
            "tabs" | "spaces" | "either" => Ok(()),
            _ => Err(garde::Error::new(format!(
                "'{v}' is not valid - use 'tabs', 'spaces', or 'either'"
            ))),
        }
    } else {
        Ok(())
    }
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_newline_policy(value: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        match v.as_str() {
            "require" | "allow" => Ok(()),
            _ => Err(garde::Error::new(format!(
                "'{v}' is not valid - use 'require' or 'allow'"
            ))),
        }
    } else {
        Ok(())
    }
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_line_endings(value: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        match v.as_str() {
            "lf" | "crlf" | "either" => Ok(()),
            _ => Err(garde::Error::new(format!(
                "'{v}' is not valid - use 'lf', 'crlf', or 'either'"
            ))),
        }
    } else {
        Ok(())
    }
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_tab_width(value: &Option<usize>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        if *v >= 1 && *v <= 16 {
            Ok(())
        } else {
            Err(garde::Error::new(format!(
                "{v} is outside the range 1-16 - common values are 2, 4, or 8"
            )))
        }
    } else {
        Ok(())
    }
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_max_line_length(value: &Option<usize>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        if *v >= 40 && *v <= 500 {
            Ok(())
        } else {
            Err(garde::Error::new(format!(
                "{v} is outside the range 40-500 - common values are 80, 100, or 120"
            )))
        }
    } else {
        Ok(())
    }
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_max_lines(value: &Option<usize>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        if *v >= 50 && *v <= 5000 {
            Ok(())
        } else {
            Err(garde::Error::new(format!(
                "{v} is outside the range 50-5000 - common values are 300, 400, or 500"
            )))
        }
    } else {
        Ok(())
    }
}

// ============================================================================
// Config Error
// ============================================================================

/// Error type for configuration loading
#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "config read error: {e}"),
            Self::Parse(e) => write!(f, "config parse error: {e}"),
            Self::Validation(e) => write!(f, "config validation error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

// ============================================================================
// Config Loading
// ============================================================================

impl DictateConfig {
    /// Load configuration from a TOML file with validation.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::Io` if the file cannot be read.
    /// Returns `ConfigError::Parse` if the TOML content is invalid.
    /// Returns `ConfigError::Validation` if decree settings fail validation.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;

        let config: Self =
            toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;

        // Validate all decree settings
        for (name, settings) in &config.decree {
            settings
                .validate()
                .map_err(|e| ConfigError::Validation(format!("decree.{name}: {e}")))?;
        }

        Ok(config)
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

    /// Load from default location, returning error details on failure.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` with details if loading or validation fails.
    pub fn load_default_strict() -> Result<Option<Self>, ConfigError> {
        let cwd = std::env::current_dir().map_err(|e| ConfigError::Io(e.to_string()))?;
        let config_path = cwd.join(".dictate.toml");

        if !config_path.exists() {
            return Ok(None);
        }

        Self::from_file(&config_path).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_config() {
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

        // Validate all decrees
        for (name, settings) in &config.decree {
            settings.validate().unwrap_or_else(|e| {
                panic!("decree.{name} validation failed: {e}");
            });
        }

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

    #[test]
    fn rejects_invalid_max_line_length() {
        let settings = DecreeSettings {
            max_line_length: Some(10), // Too small (min 40)
            ..Default::default()
        };

        let result = settings.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("40-500"));
    }

    #[test]
    fn rejects_negative_max_line_length_at_parse() {
        // Negative values fail at TOML parse for usize
        let toml = r#"
[decree.supreme]
max_line_length = -340
"#;
        let result: Result<DictateConfig, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_tabs_vs_spaces() {
        let settings = DecreeSettings {
            tabs_vs_spaces: Some("tab".to_string()), // Should be "tabs"
            ..Default::default()
        };

        let result = settings.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("tabs"));
    }

    #[test]
    fn rejects_invalid_line_endings() {
        let settings = DecreeSettings {
            line_endings: Some("windows".to_string()), // Should be "crlf"
            ..Default::default()
        };

        let result = settings.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("lf"));
    }

    #[test]
    fn rejects_tab_width_out_of_range() {
        let settings = DecreeSettings {
            tab_width: Some(32), // Max is 16
            ..Default::default()
        };

        let result = settings.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("1-16"));
    }

    #[test]
    fn rejects_max_lines_out_of_range() {
        let settings = DecreeSettings {
            max_lines: Some(10), // Min is 50
            ..Default::default()
        };

        let result = settings.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("50-5000"));
    }

    #[test]
    fn accepts_valid_settings() {
        let settings = DecreeSettings {
            trailing_whitespace: Some("deny".to_string()),
            tabs_vs_spaces: Some("spaces".to_string()),
            tab_width: Some(4),
            final_newline: Some("require".to_string()),
            line_endings: Some("lf".to_string()),
            max_line_length: Some(100),
            blank_line_whitespace: Some("allow".to_string()),
            max_lines: Some(500),
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn accepts_none_values() {
        let settings = DecreeSettings::default();
        assert!(settings.validate().is_ok());
    }
}
