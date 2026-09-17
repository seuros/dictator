#![warn(rust_2024_compatibility, clippy::all)]

//! decree.python - Python structural rules (PEP 8 compliant).

mod file_length;
mod imports;
mod indentation;

use dictator_decree_abi::{BoxDecree, Decree, Diagnostics};
use dictator_supreme::SupremeConfig;

pub use imports::{ImportType, classify_module, is_python_stdlib};

/// Configuration for python decree
#[derive(Debug, Clone)]
pub struct PythonConfig {
    pub max_lines: usize,
    pub ignore_comments: bool,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            max_lines: file_length::DEFAULT_MAX_LINES,
            ignore_comments: false,
        }
    }
}

#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_config(source, &PythonConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &PythonConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    file_length::check_file_line_count(source, config.max_lines, &mut diags);
    imports::check_import_ordering(source, &mut diags);
    indentation::check_indentation_consistency(source, &mut diags);

    diags
}

#[derive(Default)]
pub struct Python {
    config: PythonConfig,
    supreme: SupremeConfig,
}

impl Python {
    #[must_use]
    pub const fn new(config: PythonConfig, supreme: SupremeConfig) -> Self {
        Self { config, supreme }
    }
}

impl Decree for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        let mut diags = Diagnostics::new();

        let supreme_diags =
            dictator_supreme::lint_source_with_owner(source, &self.supreme, "python");

        if self.config.ignore_comments {
            diags.extend(dictator_supreme::retain_long_line_diags(
                source,
                supreme_diags,
                "python/line-too-long",
                "#",
            ));
        } else {
            diags.extend(supreme_diags);
        }

        diags.extend(lint_source_with_config(source, &self.config));
        diags
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Python structural rules".to_string(),
            persona: "Monty".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["py".to_string(), "pyi".to_string(), "pyw".to_string()],
            supported_filenames: vec![
                "pyproject.toml".to_string(),
                "setup.py".to_string(),
                "setup.cfg".to_string(),
                "Pipfile".to_string(),
                "requirements.txt".to_string(),
                "requirements-dev.txt".to_string(),
                "constraints.txt".to_string(),
                ".python-version".to_string(),
                "pyrightconfig.json".to_string(),
                "mypy.ini".to_string(),
            ],
            skip_filenames: vec![
                "Pipfile.lock".to_string(),
                "poetry.lock".to_string(),
                "uv.lock".to_string(),
                "pdm.lock".to_string(),
            ],
            file_scope_rules: dictator_supreme::file_scope_rules(&["file-too-long"]),
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(Python::default())
}

/// Create decree with custom config
#[must_use]
pub fn init_decree_with_config(config: PythonConfig) -> BoxDecree {
    Box::new(Python::new(config, SupremeConfig::default()))
}

/// Create decree with custom config + supreme config (merged from decree.supreme + decree.python)
#[must_use]
pub fn init_decree_with_configs(config: PythonConfig, supreme: SupremeConfig) -> BoxDecree {
    Box::new(Python::new(config, supreme))
}

/// Convert `DecreeSettings` to `PythonConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> PythonConfig {
    PythonConfig {
        max_lines: settings.max_lines.unwrap_or(file_length::DEFAULT_MAX_LINES),
        ignore_comments: settings.ignore_comments.unwrap_or(false),
    }
}

#[cfg(test)]
mod tests;
