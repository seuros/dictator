//! Configuration loading for .dictate.toml

use crate::error::{DictatorContext, suggestions};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-rule ignore configuration.
///
/// Ignores are evaluated by the host (Regime) after decrees emit diagnostics.
/// The match is based on:
/// - `filenames`: exact filename match (e.g. `Makefile`)
/// - `extensions`: file extension match (e.g. `md`, `mdx`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RuleIgnore {
    /// Exact filenames to ignore this rule for.
    #[serde(default)]
    pub filenames: Vec<String>,
    /// File extensions to ignore this rule for (case-insensitive).
    #[serde(default)]
    pub extensions: Vec<String>,
}

/// Configuration profile for different environments (strict, relaxed, etc.)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProfileConfig {
    /// Inherit from another profile (optional)
    pub inherits: Option<String>,

    /// Override decree settings for this profile
    #[serde(default)]
    pub decree: HashMap<String, DecreeSettings>,

    /// Profile description
    pub description: Option<String>,
}

/// Root configuration from .dictate.toml
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DictateConfig {
    #[serde(default)]
    pub decree: HashMap<String, DecreeSettings>,

    /// Configuration profiles for different environments
    #[serde(default)]
    pub profile: HashMap<String, ProfileConfig>,

    /// Active profile (defaults to "default" if not specified)
    pub active_profile: Option<String>,
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

    // Rust-specific settings
    #[garde(custom(validate_rust_edition))]
    pub min_edition: Option<String>,
    #[garde(custom(validate_rust_version))]
    pub min_rust_version: Option<String>,

    // Frontmatter decree settings
    #[garde(skip)]
    pub order: Option<Vec<String>>,
    #[garde(skip)]
    pub required: Option<Vec<String>>,

    // Linter integration (for supremecourt mode)
    #[garde(skip)]
    pub linter: Option<LinterConfig>,

    /// Ignore specific rules for specific files/extensions.
    ///
    /// Example:
    /// ```toml
    /// [decree.supreme.ignore.tab-character]
    /// filenames = ["Makefile"]
    /// extensions = ["md", "mdx"]
    /// ```
    #[serde(default)]
    #[garde(skip)]
    pub ignore: HashMap<String, RuleIgnore>,
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

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::option_if_let_else
)]
fn validate_rust_edition(value: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(v) = value {
        match v.as_str() {
            "2015" | "2018" | "2021" | "2024" => Ok(()),
            _ => Err(garde::Error::new(format!(
                "'{v}' is not a valid Rust edition - use '2015', '2018', '2021', or '2024'"
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
fn validate_rust_version(value: &Option<String>, _ctx: &()) -> garde::Result {
    let Some(v) = value else {
        return Ok(());
    };
    // Accept semver-like versions: 1.70, 1.70.0, 1.83.1
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(garde::Error::new(format!(
            "'{v}' is not a valid Rust version - use format like '1.83' or '1.83.0'"
        )));
    }
    for part in parts {
        if part.parse::<u32>().is_err() {
            return Err(garde::Error::new(format!(
                "'{v}' is not a valid Rust version - use format like '1.83' or '1.83.0'"
            )));
        }
    }
    Ok(())
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
        let path_buf = path.to_path_buf();
        let content = std::fs::read_to_string(path)
            .config_context(
                path_buf.clone(),
                None,
                suggestions::config_suggestions("file_not_found"),
            )
            .map_err(|e| ConfigError::Io(e.to_string()))?;

        let config: Self = toml::from_str(&content)
            .config_context(
                path_buf,
                None,
                suggestions::config_suggestions("invalid_toml"),
            )
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        config.validate_all_settings()?;

        Ok(config)
    }

    /// Load from default location (.dictate.toml in current directory)
    #[must_use]
    pub fn load_default() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        Self::load_from_dir(&cwd)
    }

    /// Load `.dictate.toml` from a specific directory, bypassing the process cwd.
    ///
    /// Lets MCP handlers honor a caller-supplied `workspace` override instead of
    /// always reading the server process's own working directory.
    #[must_use]
    pub fn load_from_dir(dir: &std::path::Path) -> Option<Self> {
        let config_path = dir.join(".dictate.toml");

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

    /// Get the effective configuration for a specific profile
    ///
    /// This resolves profile inheritance and merges profile-specific settings
    /// with the base configuration.
    pub fn get_profile_config(&self, profile_name: &str) -> Result<Self, String> {
        let mut result = self.clone();

        // Resolve parent profiles first so the requested profile remains most specific.
        let mut current_profile = profile_name;
        let mut visited_profiles = std::collections::HashSet::new();
        let mut inheritance_chain = Vec::new();

        loop {
            if !visited_profiles.insert(current_profile.to_string()) {
                return Err(format!(
                    "Circular profile inheritance detected involving '{current_profile}'"
                ));
            }

            let Some(profile) = self.profile.get(current_profile) else {
                return Err(format!("Profile '{current_profile}' not found"));
            };
            inheritance_chain.push(profile);

            if let Some(parent) = &profile.inherits {
                current_profile = parent;
            } else {
                break;
            }
        }

        for profile in inheritance_chain.iter().rev() {
            // Merge profile decree settings
            for (decree_name, profile_settings) in &profile.decree {
                if let Some(base_settings) = result.decree.get_mut(decree_name) {
                    // Merge settings (profile overrides base)
                    merge_decree_settings(base_settings, profile_settings);
                } else {
                    // Add new decree settings from profile
                    result
                        .decree
                        .insert(decree_name.clone(), profile_settings.clone());
                }
            }
        }

        Ok(result)
    }

    /// Get the active profile configuration
    #[must_use]
    pub fn get_active_profile_config(&self) -> Self {
        let profile_name = self.active_profile.as_deref().unwrap_or("default");

        if self.profile.contains_key(profile_name) {
            self.get_profile_config(profile_name).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Failed to resolve profile '{profile_name}': {e}. \
                     Using base configuration."
                );
                self.clone()
            })
        } else {
            // No active profile or profile doesn't exist, use base config
            self.clone()
        }
    }

    fn validate_all_settings(&self) -> Result<(), ConfigError> {
        for (name, settings) in &self.decree {
            settings
                .validate()
                .map_err(|e| ConfigError::Validation(format!("decree.{name}: {e}")))?;
        }

        for (profile_name, profile) in &self.profile {
            for (name, settings) in &profile.decree {
                settings.validate().map_err(|e| {
                    ConfigError::Validation(format!("profile.{profile_name}.decree.{name}: {e}"))
                })?;
            }
        }

        Ok(())
    }
}

/// Merge profile settings into base settings (profile overrides base)
fn merge_decree_settings(base: &mut DecreeSettings, profile: &DecreeSettings) {
    // Override each field if profile provides a value
    if profile.enabled.is_some() {
        base.enabled = profile.enabled;
    }
    if profile.path.is_some() {
        base.path.clone_from(&profile.path);
    }
    if profile.trailing_whitespace.is_some() {
        base.trailing_whitespace
            .clone_from(&profile.trailing_whitespace);
    }
    if profile.tabs_vs_spaces.is_some() {
        base.tabs_vs_spaces.clone_from(&profile.tabs_vs_spaces);
    }
    if profile.tab_width.is_some() {
        base.tab_width = profile.tab_width;
    }
    if profile.final_newline.is_some() {
        base.final_newline.clone_from(&profile.final_newline);
    }
    if profile.line_endings.is_some() {
        base.line_endings.clone_from(&profile.line_endings);
    }
    if profile.max_line_length.is_some() {
        base.max_line_length = profile.max_line_length;
    }
    if profile.blank_line_whitespace.is_some() {
        base.blank_line_whitespace
            .clone_from(&profile.blank_line_whitespace);
    }
    if profile.max_lines.is_some() {
        base.max_lines = profile.max_lines;
    }
    if profile.ignore_comments.is_some() {
        base.ignore_comments = profile.ignore_comments;
    }
    if profile.ignore_blank_lines.is_some() {
        base.ignore_blank_lines = profile.ignore_blank_lines;
    }
    if profile.method_visibility_order.is_some() {
        base.method_visibility_order
            .clone_from(&profile.method_visibility_order);
    }
    if profile.comment_spacing.is_some() {
        base.comment_spacing = profile.comment_spacing;
    }
    if profile.import_order.is_some() {
        base.import_order.clone_from(&profile.import_order);
    }
    if profile.visibility_order.is_some() {
        base.visibility_order.clone_from(&profile.visibility_order);
    }
    if profile.min_edition.is_some() {
        base.min_edition.clone_from(&profile.min_edition);
    }
    if profile.min_rust_version.is_some() {
        base.min_rust_version.clone_from(&profile.min_rust_version);
    }
    if profile.order.is_some() {
        base.order.clone_from(&profile.order);
    }
    if profile.required.is_some() {
        base.required.clone_from(&profile.required);
    }
    if profile.linter.is_some() {
        base.linter.clone_from(&profile.linter);
    }
    // Merge ignore rules (profile adds to base)
    for (rule, ignore) in &profile.ignore {
        base.ignore.insert(rule.clone(), ignore.clone());
    }
}

#[cfg(test)]
mod tests;
